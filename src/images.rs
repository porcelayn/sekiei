use crate::config::Config;
use crate::file_ops::create_directory_safely;
use image::{
    self, ImageEncoder, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder,
    codecs::webp::WebPEncoder, imageops,
};
use std::error::Error;
use std::fs;
use std::path::Path;
use walkdir::DirEntry;
use colored::Colorize;

pub fn create_placeholder_image(
    img_path: &Path,
    output_path: &Path,
    use_webp: bool,
) -> Result<(), Box<dyn Error>> {
    let img = image::open(img_path)?;

    let width = 20;
    let height = (img.height() as f32 * (width as f32 / img.width() as f32)) as u32;

    let tiny = img.resize(width, height, imageops::FilterType::Triangle);
    let blurred = tiny.blur(3.0);

    if let Some(parent) = output_path.parent() {
        create_directory_safely(parent)?;
    }

    let mut buffer = Vec::new();

    if use_webp {
        let encoder = WebPEncoder::new_lossless(&mut buffer);
        encoder.encode(
            blurred.as_bytes(),
            blurred.width(),
            blurred.height(),
            blurred.color().into(),
        )?;
    } else if output_path.extension().and_then(|e| e.to_str()) == Some("jpg")
        || output_path.extension().and_then(|e| e.to_str()) == Some("jpeg")
    {
        let mut encoder = JpegEncoder::new_with_quality(&mut buffer, 30);
        encoder.encode_image(&blurred)?;
    } else {
        let encoder = PngEncoder::new_with_quality(
            &mut buffer,
            image::codecs::png::CompressionType::Fast,
            image::codecs::png::FilterType::NoFilter,
        );
        encoder.write_image(
            blurred.as_bytes(),
            blurred.width(),
            blurred.height(),
            blurred.color().into(),
        )?;
    }

    fs::write(output_path, buffer)?;
    Ok(())
}

pub fn process_content_images(
    entry: &DirEntry,
    dist_static: &Path,
    lazy_dir: &Path,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    let relative_path = entry.path().strip_prefix("content")?;
    let sanitized_name = crate::utils::sanitize_filename(&relative_path.to_string_lossy());
    let mut output_path = dist_static.join(&sanitized_name);
    create_directory_safely(output_path.parent().unwrap())?;

    match entry.path().extension().and_then(|s| s.to_str().map(|s| s.to_lowercase())) {
        Some(ext) if (ext == "jpg" || ext == "jpeg" || ext == "png") && config.images.compress_to_webp => {
            let img = image::open(entry.path())?;
            let rgba_img = img.to_rgba8();
            let mut buffer = Vec::new();
            let encoder = WebPEncoder::new_lossless(&mut buffer);
            encoder.encode(
                rgba_img.as_raw(),
                rgba_img.width(),
                rgba_img.height(),
                image::ExtendedColorType::Rgba8,
            )?;

            output_path.set_extension("webp");
            fs::write(&output_path, &buffer)?;

            let file_stem = output_path.file_stem().unwrap_or_default().to_string_lossy();
            let placeholder_path = lazy_dir.join(format!("{}.webp", file_stem));
            create_placeholder_image(entry.path(), &placeholder_path, true)?;

            println!(
                "{} {} -> {} (WebP) with placeholder",
                "Converting".green(),
                entry.path().display().to_string().replace('\\', "/").yellow(),
                output_path.display().to_string().replace('\\', "/").yellow()
            );
        }
        Some(ext) if ext == "jpg" || ext == "jpeg" => {
            let img = image::open(entry.path())?;
            let quality = config.images.quality.min(100);
            let mut buffer = Vec::new();
            let mut encoder = JpegEncoder::new_with_quality(&mut buffer, quality);
            encoder.encode_image(&img)?;

            fs::write(&output_path, &buffer)?;

            let file_stem = output_path.file_stem().unwrap_or_default().to_string_lossy();
            let placeholder_path = lazy_dir.join(format!("{}.jpg", file_stem));
            create_placeholder_image(entry.path(), &placeholder_path, false)?;

            println!(
                "{} {} -> {} (quality: {}) with placeholder",
                "Compressing".green(),
                entry.path().display().to_string().replace('\\', "/").yellow(),
                output_path.display().to_string().replace('\\', "/").yellow(),
                quality.to_string().cyan()
            );
        }
        Some(ext) if ext == "png" => {
            let img = image::open(entry.path())?;
            let quality = config.images.quality.min(100);
            let mut buffer = Vec::new();
            let compression = match quality {
                0..=33 => image::codecs::png::CompressionType::Fast,
                34..=66 => image::codecs::png::CompressionType::Default,
                67..=100 => image::codecs::png::CompressionType::Best,
                _ => unreachable!("Quality capped at 100"),
            };
            let encoder = PngEncoder::new_with_quality(
                &mut buffer,
                compression,
                image::codecs::png::FilterType::NoFilter,
            );
            encoder.write_image(
                img.as_bytes(),
                img.width(),
                img.height(),
                image::ExtendedColorType::Rgba8,
            )?;

            fs::write(&output_path, &buffer)?;

            let file_stem = output_path.file_stem().unwrap_or_default().to_string_lossy();
            let placeholder_path = lazy_dir.join(format!("{}.png", file_stem));
            create_placeholder_image(entry.path(), &placeholder_path, false)?;

            println!(
                "{} {} -> {} (quality: {}) with placeholder",
                "Compressing".green(),
                entry.path().display().to_string().yellow().replace('\\', "/").yellow(),
                output_path.display().to_string().yellow().replace('\\', "/").yellow(),
                quality.to_string().cyan()
            );
        }
        _ => {
            fs::copy(entry.path(), &output_path)?;
            println!(
                "{} {} -> {}",
                "Copying".green(),
                entry.path().display().to_string().yellow().replace('\\', "/").yellow(),
                output_path.display().to_string().yellow().replace('\\', "/").yellow()
            );
        }
    }
    Ok(())
}