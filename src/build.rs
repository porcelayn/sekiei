use std::fs::{self, OpenOptions};
use std::path::Path;
use std::io::Write;
use tera::{Tera, Context};
use pulldown_cmark::{Parser, html};
use walkdir::WalkDir;
use serde_yaml::Value as YamlValue;
use std::error::Error;

fn extract_frontmatter(content: &str) -> Result<(YamlValue, &str), Box<dyn Error>> {
    let trimmed_content = content.trim_start();
    
    if !trimmed_content.starts_with("---") {
        return Err("Frontmatter is missing".into());
    }
    
    let end_pattern = "\n---";
    if let Some(end) = trimmed_content[3..].find(end_pattern) {
        let frontmatter_end = 3 + end;
        let frontmatter_str = &trimmed_content[3..frontmatter_end].trim();
        
        let frontmatter: YamlValue = serde_yaml::from_str(frontmatter_str)?;
        
        if frontmatter.get("title").is_none() {
            return Err("Missing title in frontmatter".into());
        }
        if frontmatter.get("date").is_none() {
            return Err("Missing date in frontmatter".into());
        }
        
        if !frontmatter["title"].is_string() {
            return Err("Title must be a string".into());
        }
        if !frontmatter["date"].is_string() {
            return Err("Date must be a string".into());
        }
        
        let md_content = &trimmed_content[frontmatter_end + end_pattern.len()..];
        
        Ok((frontmatter, md_content))
    } else {
        eprintln!("Content starts with: {}", &trimmed_content[..50.min(trimmed_content.len())]);
        eprintln!("Could not find end pattern '\\n---' in content");
        Err("Frontmatter end delimiter not found".into())
    }
}

fn safely_create_directory(path: &Path) -> Result<(), Box<dyn Error>> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| {
            eprintln!("Failed to create directory {}: {}", path.display(), e);
            e
        })?;
    }
    Ok(())
}

fn safely_write_file(path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        safely_create_directory(parent)?;
    }
    
    // Open file with explicit permissions
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(|e| {
            eprintln!("Failed to open file for writing {}: {}", path.display(), e);
            e
        })?;
    
    // Write content
    file.write_all(content.as_bytes()).map_err(|e| {
        eprintln!("Failed to write to file {}: {}", path.display(), e);
        e
    })?;
    
    Ok(())
}

fn safely_copy_file(src: &Path, dest: &Path) -> Result<(), Box<dyn Error>> {
    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        safely_create_directory(parent)?;
    }
    
    // Read source file
    let content = fs::read_to_string(src).map_err(|e| {
        eprintln!("Failed to read file {}: {}", src.display(), e);
        e
    })?;
    
    // Write to destination
    safely_write_file(dest, &content)?;
    
    Ok(())
}

fn safely_clear_directory(dir: &Path) -> Result<(), Box<dyn Error>> {
    if !dir.exists() {
        return Ok(());
    }
    
    println!("Clearing directory: {}", dir.display());
    
    let mut entries = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        entries.push(entry);
    }
    
    for entry in &entries {
        if entry.path().is_file() {
            println!("Removing file: {}", entry.path().display());
            if let Err(e) = fs::remove_file(entry.path()) {
                eprintln!("Warning: Failed to remove file {}: {}", entry.path().display(), e);
            }
        }
    }
    
    for entry in entries.iter().rev() {
        if entry.path().is_dir() && entry.path() != dir {
            println!("Removing directory: {}", entry.path().display());
            if let Err(e) = fs::remove_dir(entry.path()) {
                eprintln!("Warning: Failed to remove directory {}: {}", entry.path().display(), e);
            }
        }
    }
    
    Ok(())
}

pub fn build() -> Result<(), Box<dyn Error>> {
    let dist = Path::new("dist");
    
    safely_clear_directory(dist)?;
    safely_create_directory(dist)?;
    
    println!("Loading templates...");
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error parsing templates: {}", e);
            return Err(e.into());
        }
    };
    
    println!("Processing content files...");
    for entry in WalkDir::new("content").into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
            println!("Processing markdown file: {}", entry.path().display());
            
            let file_content = match fs::read_to_string(entry.path()) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error reading file {}: {}", entry.path().display(), e);
                    continue; 
                }
            };
            
            let (frontmatter, md_content) = match extract_frontmatter(&file_content) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Error in {}: {}", entry.path().display(), e);
                    continue; 
                }
            };
            
            let parser = Parser::new(md_content);
            let mut html_content = String::new();
            html::push_html(&mut html_content, parser);
            
            let title = frontmatter["title"].as_str().unwrap().to_string();
            
            let mut context = Context::new();
            context.insert("title", &title);
            context.insert("content", &html_content);
            context.insert("frontmatter", &frontmatter);
            
            let relative_path = match entry.path().strip_prefix("content") {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Error getting relative path for {}: {}", entry.path().display(), e);
                    continue;
                }
            };
            
            let output_path = if relative_path == Path::new("index.md") {
                dist.join("index.html")
            } else {
                let output_dir = dist.join(relative_path.with_extension(""));
                safely_create_directory(&output_dir)?;
                output_dir.join("index.html")
            };
            
            println!("Rendering to: {}", output_path.display());
            let rendered = match tera.render("base.html", &context) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error rendering template for {}: {}", entry.path().display(), e);
                    continue;
                }
            };
            
            if let Err(e) = safely_write_file(&output_path, &rendered) {
                eprintln!("Error writing to {}: {}", output_path.display(), e);
                continue;
            }
        } else if entry.path().is_file() {
            let relative_path = match entry.path().strip_prefix("content") {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Error getting relative path for {}: {}", entry.path().display(), e);
                    continue;
                }
            };
            
            let output_path = dist.join(relative_path);
            println!("Copying file to: {}", output_path.display());
            
            if let Err(e) = safely_copy_file(entry.path(), &output_path) {
                eprintln!("Error copying {} to {}: {}", entry.path().display(), output_path.display(), e);
                continue;
            }
        }
    }
    
    let static_dir = Path::new("static");
    if static_dir.exists() {
        println!("Copying static files...");
        for entry in WalkDir::new(static_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                let relative_path = match entry.path().strip_prefix(static_dir) {
                    Ok(path) => path,
                    Err(e) => {
                        eprintln!("Error getting relative path for {}: {}", entry.path().display(), e);
                        continue;
                    }
                };
                
                let dest_path = dist.join("static").join(relative_path);
                println!("Copying static file to: {}", dest_path.display());
                
                if let Err(e) = safely_copy_file(entry.path(), &dest_path) {
                    eprintln!("Error copying static file {} to {}: {}", entry.path().display(), dest_path.display(), e);
                    continue;
                }
            }
        }
    }
    
    println!("Build completed successfully!");
    Ok(())
}