use crate::file_ops::{create_directory_safely, safely_write_file};
use std::error::Error;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use css_minify::optimizations::{Level as CssLevel, Minifier as CssMinifier};
use minify_js::{Session, TopLevelMode, minify as js_minify};
use colored::Colorize;

pub fn process_static_files(dist_static: &Path) -> Result<(), Box<dyn Error>> {
    let static_dir = Path::new("static");
    if static_dir.exists() {
        for entry in WalkDir::new(static_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                let relative_path = entry.path().strip_prefix(static_dir)?;
                let output_path = dist_static.join(relative_path);
                create_directory_safely(output_path.parent().unwrap())?;

                match entry.path().extension().and_then(|s| s.to_str()) {
                    Some("css") => {
                        let css_content = fs::read_to_string(entry.path())?;
                        let minified_css = CssMinifier::default()
                            .minify(&css_content, CssLevel::Three).expect("Failed to minify CSS");
                        safely_write_file(&output_path, &minified_css)?;
                        println!(
                            "{} {} -> {}",
                            "Copying and minifying".green(),
                            entry.path().display().to_string().yellow(),
                            output_path.display().to_string().yellow()
                        );
                    }
                    Some("js") => {
                        let js_content = fs::read(entry.path())?;
                        let mut minified_js = Vec::new();
                        let js_session = Session::new();
                        js_minify(
                            &js_session,
                            TopLevelMode::Global,
                            &js_content,
                            &mut minified_js,
                        ).expect("Failed to minify JS");
                        fs::write(&output_path, &minified_js)?;
                        println!(
                            "{} {} -> {}",
                            "Copying and minifying".green(),
                            entry.path().display().to_string().yellow(),
                            output_path.display().to_string().yellow()
                        );
                    }
                    _ => {
                        fs::copy(entry.path(), &output_path)?;
                        println!(
                            "{} {} -> {}",
                            "Copying".green(),
                            entry.path().display().to_string().yellow(),
                            output_path.display().to_string().yellow()
                        );
                    }
                }
            }
        }
    } else {
        println!("{}", "No static folder found, skipping static file copy.".yellow());
    }
    Ok(())
}