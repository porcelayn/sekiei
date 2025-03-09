use minify_html::minify;
use serde::Serialize;
use serde_yaml::Value as YamlValue;
use std::{error::Error, fs, path::Path};
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
        eprintln!(
            "Content starts with: {}",
            &trimmed_content[..50.min(trimmed_content.len())]
        );
        eprintln!("Could not find end pattern '\\n---' in content");
        Err("Frontmatter end delimiter not found".into())
    }
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
            let rel_path = path.with_extension("").strip_prefix("content")?.to_string_lossy().to_string();
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
            let url = format!("/static/{}", rel_path);

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

fn markdown_to_html(markdown: &str) -> String {
    let mut html = String::new();
    let options =
        pulldown_cmark::Options::ENABLE_GFM | pulldown_cmark::Options::ENABLE_STRIKETHROUGH;
    let parser: pulldown_cmark::Parser<'_> = pulldown_cmark::Parser::new_ext(markdown, options);
    pulldown_cmark::html::push_html(&mut html, parser);
    html
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

    println!("Loading Markdown files from content/");

    for entry in WalkDir::new("content").into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("md")
        {
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
            let html_content = markdown_to_html(md_content);

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

            if let Err(e) =
                safely_write_file(&output_path, String::from_utf8(minified).unwrap().as_str())
            {
                eprintln!("Error writing to {}: {}", output_path.display(), e);
                continue;
            }

            println!(
                "Converting {} -> {}",
                entry.path().display(),
                output_path.display()
            );
        } else if entry.path().is_dir() && entry.path().display().to_string() != "content" {
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

            if let Err(e) =
                safely_write_file(&output_dir.join("index.html"), String::from_utf8(minified).unwrap().as_str())
            {
                eprintln!("Error writing to {}: {}", output_dir.display(), e);
                continue;
            }

            println!(
                "Creating listing for {} -> {}",
                entry.path().display(),
                output_dir.display()
            );
        }
    }

    Ok(())
}
