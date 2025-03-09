use minify_html::minify;
use regex::Regex;
use serde::Serialize;
use serde_yaml::Value as YamlValue;
use std::{error::Error, fs, path::{Path, PathBuf}};
use tera::Tera;
use walkdir::WalkDir;

fn clear_directory_safely(path: &std::path::Path) -> std::io::Result<()> {
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    std::fs::create_dir(path)?;
    Ok(())
}

fn create_directory_safely(path: &std::path::Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

fn safely_write_file(path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
    fs::write(path, content)?;
    Ok(())
}

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
        Err("Frontmatter end delimiter not found".into())
    }
}

fn sanitize_filename(path: &str) -> String {
    let mut sanitized = String::new();
    for c in path.chars() {
        if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
            sanitized.push(c);
        } else {
            sanitized.push_str(&format!("-u{:04x}", c as u32));
        }
    }
    sanitized.replace('/', "-")
}

fn process_paths(markdown: &str, current_path: &Path) -> String {
    let markdown = process_standard_images(markdown, current_path);
    let markdown = process_alternative_images(&markdown, current_path);
    let markdown = process_links(&markdown);
    markdown
}

fn process_standard_images(markdown: &str, current_path: &Path) -> String {
    let re = Regex::new(r"!\[(.*?)\]\(([^)]+)\)").unwrap();
    
    re.replace_all(markdown, |caps: &regex::Captures| {
        let alt_text = &caps[1];
        let path = &caps[2];
        
        if !path.starts_with("http://") && !path.starts_with("https://") && !path.starts_with('/') {
            let static_path = resolve_path(path, current_path);
            format!("![{}]({})", alt_text, static_path)
        } else {
            format!("![{}]({})", alt_text, path)
        }
    }).to_string()
}

fn process_alternative_images(markdown: &str, current_path: &Path) -> String {
    let re = Regex::new(r"!\[\[([^|\]]+)(?:\|([^\]]*))?\]\]").unwrap();
    
    re.replace_all(markdown, |caps: &regex::Captures| {
        let path = &caps[1];
        let alt_text = caps.get(2).map_or("", |m| m.as_str());
        
        if !path.starts_with("http://") && !path.starts_with("https://") && !path.starts_with('/') {
            let static_path = find_unique_image(path, current_path);
            format!("![{}]({})", alt_text, static_path)
        } else {
            format!("![{}]({})", alt_text, path)
        }
    }).to_string()
}

fn find_unique_image(image_name: &str, current_path: &Path) -> String {
    if image_name.contains('/') {
        return resolve_path(image_name, current_path);
    }
    
    let mut matches = Vec::new();
    
    for entry in WalkDir::new("content").into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        
        let file_name = entry.file_name().to_string_lossy();
        if file_name == image_name {
            matches.push(entry.path().to_path_buf());
        }
    }
    
    match matches.len() {
        0 => resolve_path(image_name, current_path),
        1 => {
            let match_path = matches[0]
                .strip_prefix("content")
                .unwrap_or(&matches[0]);
            format!("/static/{}", sanitize_filename(&match_path.to_string_lossy()))
        }
        _ => {
            for dir_path in ["content"].iter() {
                for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
                    if !entry.file_type().is_file() {
                        continue;
                    }
                    
                    let file_name = entry.file_name().to_string_lossy();
                    if file_name == image_name {
                        let match_path = entry.path()
                            .strip_prefix("content")
                            .unwrap_or(entry.path());
                        return format!("/static/{}", sanitize_filename(&match_path.to_string_lossy()));
                    }
                }
            }
            resolve_path(image_name, current_path)
        }
    }
}

fn process_links(markdown: &str) -> String {
    let re = Regex::new(r"\[\[([^|\]]+)(?:\|([^\]]*))?\]\]").unwrap();
    
    re.replace_all(markdown, |caps: &regex::Captures| {
        let path = &caps[1];
        let display_text = caps.get(2).map_or_else(
            || {
                if path.starts_with("wiki:") {
                    path.strip_prefix("wiki:").unwrap_or(path)
                } else {
                    path.split('/').last().unwrap_or(path)
                }
            },
            |m| m.as_str()
        );
        
        if path.starts_with("wiki:") {
            let article = path.strip_prefix("wiki:").unwrap();
            format!("[wiki:{}](https://en.wikipedia.org/wiki/{})", display_text, article)
        } else if !path.starts_with("http://") && !path.starts_with("https://") && !path.starts_with('/') {
            let link_path = if !path.contains('/') {
                find_unique_internal_link(path)
            } else {
                get_internal_link_path(path)
            };
            format!("[{}]({})", display_text, link_path)
        } else {
            format!("[{}]({})", display_text, path)
        }
    }).to_string()
}

fn get_internal_link_path(path: &str) -> String {
    let clean_path = if path.ends_with(".md") {
        &path[0..path.len() - 3]
    } else {
        path
    };
    
    if clean_path == "index" {
        "/".to_string()
    } else {
        format!("/{}", clean_path)
    }
}

fn find_unique_internal_link(link_name: &str) -> String {
    let mut matches = Vec::new();
    
    for entry in WalkDir::new("content").into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        
        let file_name = entry.file_name().to_string_lossy();
        let file_stem = entry.path()
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
            
        if file_name.ends_with(".md") && file_stem == link_name {
            matches.push(entry.path().to_path_buf());
        }
    }
    
    match matches.len() {
        0 => get_internal_link_path(link_name),
        1 => {
            let match_path = matches[0]
                .strip_prefix("content")
                .unwrap_or(&matches[0])
                .with_extension("");
            let clean_path = match_path.to_string_lossy().replace('\\', "/");
            if clean_path == "index" {
                "/".to_string()
            } else {
                format!("/{}", clean_path)
            }
        }
        _ => {
            for dir_path in ["content"].iter() {
                for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
                    if !entry.file_type().is_file() {
                        continue;
                    }
                    
                    let file_stem = entry.path()
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                        
                    if file_stem.ends_with(".md") && file_stem == link_name {
                        let match_path = entry.path()
                            .strip_prefix("content")
                            .unwrap_or(entry.path())
                            .with_extension("");
                        return format!("/{}", match_path.to_string_lossy().replace('\\', "/"));
                    }
                }
            }
            get_internal_link_path(link_name)
        }
    }
}

fn resolve_path(path: &str, current_path: &Path) -> String {
    let current_dir = current_path.parent()
        .unwrap()
        .strip_prefix("content")
        .unwrap_or(Path::new(""));
    
    let relative_path = if path.starts_with("./") || path.starts_with("../") {
        let mut full_path = PathBuf::from(current_dir);
        
        let path_segments: Vec<&str> = path.split('/').collect();
        let mut path_iter = path_segments.iter();
        
        let first_segment = *path_iter.next().unwrap_or(&"");
        match first_segment {
            "." => {},
            ".." => {
                if full_path.parent().is_some() {
                    full_path = full_path.parent().unwrap().to_path_buf();
                }
            },
            _ => {
                full_path.push(first_segment);
            }
        }
        
        for segment in path_iter {
            full_path.push(segment);
        }
        full_path.to_string_lossy().to_string()
    } else {
        path.to_string()
    };
    
    format!("/static/{}", sanitize_filename(&relative_path))
}

fn process_wiki_parenthetical_links(markdown: &str) -> String {
    let re = Regex::new(r"\[(.*?)\]\(wiki:([^)]+)\)").unwrap();
    
    re.replace_all(markdown, |caps: &regex::Captures| {
        let display_text = &caps[1];
        let article = &caps[2];
        format!("[{}](https://en.wikipedia.org/wiki/{})", display_text, article)
    }).to_string()
}

fn markdown_to_html(markdown: &str, file_path: &Path) -> String {
    let mut processed_markdown = process_paths(markdown, file_path);
    processed_markdown = process_wiki_parenthetical_links(&processed_markdown);

    let mut html = String::new();
    let options = pulldown_cmark::Options::ENABLE_GFM | pulldown_cmark::Options::ENABLE_STRIKETHROUGH;
    let parser = pulldown_cmark::Parser::new_ext(&processed_markdown, options);
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

#[derive(Serialize)]
struct ListingItem {
    name: String,
    url: String,
    date: String,
    description: Option<String>,
}

fn create_listing(dir: &Path) -> Result<Vec<ListingItem>, Box<dyn Error>> {
    let mut items = Vec::new();
    for entry in WalkDir::new(dir).max_depth(1) {
        let e = entry.expect("Failed to read directory");
        if e.depth() == 0 {
            continue;
        }

        let path = e.path();
        let name = path
            .file_name()
            .ok_or("Failed to get file name")?
            .to_string_lossy()
            .to_string();
        if e.file_type().is_file() && name.ends_with(".md") {
            let rel_path = path
                .with_extension("")
                .strip_prefix("content")?
                .to_string_lossy()
                .to_string();
            let url = format!("/{}", rel_path);

            let content = fs::read_to_string(path)?;
            let (frontmatter, _) = extract_frontmatter(&content)?;

            items.push(ListingItem {
                name: frontmatter["title"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                url,
                date: frontmatter["date"].as_str().unwrap_or_default().to_string(),
                description: frontmatter["description"].as_str().map(|s| s.to_string()),
            });
        } else if e.file_type().is_file() {
            let rel_path = path.strip_prefix("content")?.to_string_lossy().to_string();
            let sanitized_name = sanitize_filename(&rel_path);
            let url = format!("/static/{}", sanitized_name);

            let metadata = fs::metadata(path)?;
            let modified_time = metadata.modified()?;
            let date = modified_time
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs()
                .to_string();

            items.push(ListingItem {
                name: name.clone(),
                url,
                date,
                description: None,
            });
        }
    }
    Ok(items)
}

fn is_not_hidden_dir(entry: &walkdir::DirEntry) -> bool {
    if entry.file_type().is_dir() {
        entry.file_name()
            .to_str()
            .map_or(false, |name| !name.starts_with('.'))
    } else {
        true
    }
}

pub fn build() -> Result<(), Box<dyn Error>> {
    let dist = Path::new("dist");
    clear_directory_safely(dist)?;

    create_directory_safely(dist)?;
    create_directory_safely(&dist.join("static"))?;

    println!("Loading Templates defined in templates/");
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error loading templates: {}", e);
            return Err(Box::new(e));
        }
    };

    let minify_cfg = minify_html::Cfg {
        minify_js: true,
        minify_css: true,
        ..Default::default()
    };

    println!("Loading Markdown files from content");

    let mut markdown_files = Vec::new();
    let mut directories = Vec::new();

    for entry in WalkDir::new("content")
        .into_iter()
        .filter_entry(is_not_hidden_dir)
        .filter_map(|e| e.ok())
    {
        let file_name = entry.file_name().to_string_lossy();
        if file_name.starts_with(".") {
            continue;
        }

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
                    md_stem,
                    dir_name,
                    md_parent.display()
                ).into());
            }
        }
    }

    for entry in WalkDir::new("content")
        .into_iter()
        .filter_entry(is_not_hidden_dir)
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file() {
            if entry.path().file_name().expect("Could not read file").to_string_lossy().starts_with(".") {
                continue;
            }
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

                let rendered = match tera.render("content.html", &context) {
                    Ok(content) => content,
                    Err(e) => {
                        eprintln!(
                            "Error rendering template for {}: {}",
                            entry.path().display(),
                            e
                        );
                        continue;
                    }
                };

                let minified = minify(rendered.as_bytes(), &minify_cfg);
                safely_write_file(&output_path, String::from_utf8(minified).unwrap().as_str())?;

                println!(
                    "Converting {} -> {}",
                    entry.path().display(),
                    output_path.display()
                );
            } else {
                let relative_path = entry.path().strip_prefix("content")?;
                let sanitized_name = sanitize_filename(&relative_path.to_string_lossy());
                let output_path = dist.join("static").join(&sanitized_name);
                
                create_directory_safely(output_path.parent().unwrap())?;
                fs::copy(entry.path(), &output_path)?;
                println!(
                    "Copying {} -> {}",
                    entry.path().display(),
                    output_path.display()
                );
            }
        } else if entry.path().is_dir() && entry.path().display().to_string() != "content" {
            if entry.path().file_name().expect("Could not read file").to_string_lossy().starts_with(".") {
                continue;
            }
            let relative_path = entry.path().strip_prefix("content")?;
            let output_dir = dist.join(relative_path);
            create_directory_safely(&output_dir)?;
            let items = create_listing(entry.path())?;

            let mut context = tera::Context::new();
            context.insert("items", &items);
            context.insert("dir_path", &relative_path);

            let rendered = match tera.render("listing.html", &context) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!(
                        "Error rendering template for {}: {}",
                        entry.path().display(),
                        e
                    );
                    continue;
                }
            };

            let minified = minify(rendered.as_bytes(), &minify_cfg);
            safely_write_file(&output_dir.join("index.html"), String::from_utf8(minified).unwrap().as_str())?;

            println!(
                "Creating listing for {} -> {}",
                entry.path().display(),
                output_dir.display()
            );
        }
    }

    Ok(())
}
