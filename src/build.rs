use minify_html::minify;
use std::{error::Error, fs, path::Path};
use tera::Tera;
use walkdir::WalkDir;

use crate::{
    file_ops::{clear_directory_safely, create_directory_safely, safely_write_file},
    listing::create_listing,
    markdown::{extract_frontmatter, markdown_to_html},
    utils::is_not_hidden_dir,
};

pub fn build() -> Result<(), Box<dyn Error>> {
    let dist = Path::new("dist");
    clear_directory_safely(dist)?;
    create_directory_safely(dist)?;
    create_directory_safely(&dist.join("static"))?;

    println!("Loading Templates defined in templates/");
    let tera = Tera::new("templates/**/*").map_err(|e| {
        eprintln!("Error loading templates: {}", e);
        Box::new(e) as Box<dyn Error>
    })?;

    let minify_cfg = minify_html::Cfg {
        minify_js: true,
        minify_css: true,
        ..Default::default()
    };

    println!("Loading Markdown files from content");
    let mut markdown_files = Vec::new();
    let mut directories = Vec::new();

    for entry in WalkDir::new("content").into_iter().filter_entry(is_not_hidden_dir).filter_map(|e| e.ok()) {
        let file_name = entry.file_name().to_string_lossy();
        if file_name.starts_with(".") { continue; }
        if entry.path().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
            markdown_files.push(entry.path().to_path_buf());
        } else if entry.path().is_dir() && entry.path() != Path::new("content") {
            directories.push(entry.path().to_path_buf());
        }
    }

    for md_path in &markdown_files {
        let md_stem = md_path.file_stem().unwrap().to_string_lossy();
        let md_parent = md_path.parent().unwrap();
        
        for dir_path in &directories {
            let dir_name = dir_path.file_name().unwrap().to_string_lossy();
            let dir_parent = dir_path.parent().unwrap();
            
            if md_parent == dir_parent && md_stem == dir_name {
                return Err(format!(
                    "Naming conflict detected: Markdown file '{}.md' and directory '{}' have the same name at '{}'. Please use different names.",
                    md_stem, dir_name, md_parent.display()
                ).into());
            }
        }
    }

    for entry in WalkDir::new("content").into_iter().filter_entry(is_not_hidden_dir).filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            if entry.path().file_name().expect("Could not read file").to_string_lossy().starts_with(".") { continue; }
            if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
                let relative_path = entry.path().strip_prefix("content")?;
                let output_path = if relative_path.to_string_lossy() == "index.md" {
                    dist.join("index.html")
                } else {
                    let output_dir = dist.join(relative_path.with_extension(""));
                    create_directory_safely(&output_dir)?;
                    output_dir.join("index.html")
                };

                let content = fs::read_to_string(entry.path())?;
                let (frontmatter, md_content) = extract_frontmatter(&content)?;
                let html_content = markdown_to_html(md_content, entry.path());

                let mut context = tera::Context::new();
                let title = frontmatter["title"].as_str().unwrap().to_string();
                context.insert("title", &title);
                context.insert("markdown", &html_content);
                context.insert("frontmatter", &frontmatter);

                let rendered = tera.render("content.html", &context).map_err(|e| {
                    eprintln!("Error rendering template for {}: {}", entry.path().display(), e);
                    e
                })?;
                let minified = minify(rendered.as_bytes(), &minify_cfg);
                safely_write_file(&output_path, String::from_utf8(minified).unwrap().as_str())?;

                println!("Converting {} -> {}", entry.path().display(), output_path.display());
            } else {
                let relative_path = entry.path().strip_prefix("content")?;
                let sanitized_name = crate::utils::sanitize_filename(&relative_path.to_string_lossy());
                let output_path = dist.join("static").join(&sanitized_name);
                
                create_directory_safely(output_path.parent().unwrap())?;
                fs::copy(entry.path(), &output_path)?;
                println!("Copying {} -> {}", entry.path().display(), output_path.display());
            }
        } else if entry.path().is_dir() && entry.path().display().to_string() != "content" {
            if entry.path().file_name().expect("Could not read file").to_string_lossy().starts_with(".") { continue; }
            let relative_path = entry.path().strip_prefix("content")?;
            let output_dir = dist.join(relative_path);
            create_directory_safely(&output_dir)?;
            let items = create_listing(entry.path())?;

            let mut context = tera::Context::new();
            context.insert("items", &items);
            context.insert("dir_path", &relative_path);

            let rendered = tera.render("listing.html", &context).map_err(|e| {
                eprintln!("Error rendering template for {}: {}", entry.path().display(), e);
                e
            })?;
            let minified = minify(rendered.as_bytes(), &minify_cfg);
            safely_write_file(&output_dir.join("index.html"), String::from_utf8(minified).unwrap().as_str())?;

            println!("Creating listing for {} -> {}", entry.path().display(), output_dir.display());
        }
    }

    Ok(())
}