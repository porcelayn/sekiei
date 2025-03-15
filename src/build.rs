use crate::{
    config::Config,
    file_ops::{clear_directory_safely, create_directory_safely, safely_write_file},
    images::process_content_images,
    lazy_load::{add_lazy_loading, setup_lazy_loading},
    listing::create_listing,
    markdown::{Backlink, extract_frontmatter, markdown_to_html},
    paths::{init_file_cache, process_paths},
    static_files::process_static_files,
    theme::generate_theme_css,
    utils::is_not_hidden_dir,
    rss::generate_rss,
    file_tree::{process_file_tree_assets, generate_file_tree_html}
};
use colored::Colorize;
use minify_html::minify;
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::Path;
use tera::Tera;
use walkdir::WalkDir;

pub fn build() -> Result<(), Box<dyn Error>> {
    let dist = Path::new("dist");
    println!("{}", "Starting build process...".cyan());
    clear_directory_safely(dist)?;
    create_directory_safely(dist)?;
    let dist_static = dist.join("static");
    create_directory_safely(&dist_static)?;

    let lazy_dir = dist_static.join("lazy");
    create_directory_safely(&lazy_dir)?;

    let config_str = fs::read_to_string("Config.toml")
        .map_err(|e| format!("Failed to read Config.toml: {}", e))?;
    let config: Config =
        toml::from_str(&config_str).map_err(|e| format!("Failed to parse Config.toml: {}", e))?;
    config
        .images
        .validate()
        .map_err(|e| format!("Invalid [images] configuration: {}", e))?;

    let theme_css_path = dist_static.join("theme.css");
    generate_theme_css(&config, &theme_css_path)?;

    setup_lazy_loading(&dist_static)?;
    process_file_tree_assets(&dist_static)?;
    process_static_files(&dist_static)?;

    println!("{}", "Loading Templates defined in templates".blue());
    let tera = Tera::new("templates/**/*").map_err(|e| {
        eprintln!("{}", format!("Error loading templates: {}", e).red());
        Box::new(e) as Box<dyn Error>
    })?;

    let minify_cfg = minify_html::Cfg {
        minify_js: false,
        minify_css: true,
        ..Default::default()
    };

    init_file_cache();
    generate_rss(dist, &config)?;

    let file_tree_html = generate_file_tree_html(&config)?;

    let mut backlink_map: HashMap<String, HashSet<(String, String)>> = HashMap::new();
    println!("{}", "Collecting backlinks...".blue());
    for entry in WalkDir::new("content")
        .into_iter()
        .filter_entry(is_not_hidden_dir)
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("md")
        {
            let content = fs::read_to_string(entry.path())?;
            let (frontmatter, md_content) = extract_frontmatter(&content)?;
            let source_path = entry
                .path()
                .strip_prefix("content")?
                .to_string_lossy()
                .replace('\\', "/");
            let source_title = frontmatter["title"]
                .as_str()
                .unwrap_or("Untitled")
                .to_string();

            let processed_content = process_paths(md_content, entry.path());

            let mut options = Options::empty();
            options.insert(Options::ENABLE_GFM);
            let parser = Parser::new_ext(&processed_content, options);

            for event in parser {
                if let Event::Start(Tag::Link { ref dest_url, .. }) = event {
                    if !dest_url.starts_with("http") && !dest_url.starts_with("wiki:") {
                        let target_path = dest_url
                            .trim_start_matches('/')
                            .replace('\\', "/")
                            .replace(".md", "");

                        let clean_source_path = if source_path == "index.md" {
                            "/".to_string()
                        } else {
                            format!("/{}", source_path.replace(".md", ""))
                        };

                        backlink_map
                            .entry(target_path)
                            .or_insert_with(HashSet::new)
                            .insert((source_title.clone(), clean_source_path));
                    }
                }
            }
        }
    }

    for entry in WalkDir::new("content")
        .into_iter()
        .filter_entry(is_not_hidden_dir)
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file() {
            let file_name = entry.file_name().to_string_lossy();
            if file_name.starts_with(".") {
                continue;
            }

            if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
                let relative_path = entry
                    .path()
                    .strip_prefix("content")?
                    .to_string_lossy()
                    .replace('\\', "/");
                let rel_path = Path::new(&relative_path);
                let output_path = if relative_path == "index.md" {
                    dist.join("index.html")
                } else {
                    let output_dir = dist.join(rel_path.with_extension(""));
                    create_directory_safely(&output_dir)?;
                    output_dir.join("index.html")
                };

                let content = fs::read_to_string(entry.path())?;
                let (frontmatter, md_content) = extract_frontmatter(&content)?;
                let (mut html_content, toc) = markdown_to_html(md_content, entry.path());
                html_content = add_lazy_loading(&html_content, config.images.compress_to_webp);
                if config.images.compress_to_webp {
                    html_content = html_content
                        .replace(".jpg", ".webp")
                        .replace(".jpeg", ".webp")
                        .replace(".png", ".webp");
                }

                let mut context = tera::Context::new();
                let title = frontmatter["title"]
                    .as_str()
                    .unwrap_or("Untitled")
                    .to_string();
                context.insert("title", &title);
                context.insert("markdown", &html_content);
                context.insert("frontmatter", &frontmatter);
                context.insert("table_of_contents", &toc);
                context.insert("has_images", &html_content.contains("<img"));
                context.insert("file_tree", &file_tree_html);

                let current_path = relative_path.replace(".md", "");
                let clean_current_path = if current_path == "index" {
                    "".to_string()
                } else {
                    current_path
                };
                let backlinks: Vec<Backlink> = backlink_map
                    .get(&clean_current_path)
                    .unwrap_or(&HashSet::new())
                    .iter()
                    .map(|(title, path)| Backlink {
                        title: title.clone(),
                        path: path.clone(),
                    })
                    .collect();
                context.insert("backlinks", &backlinks);

                let rendered = tera.render("content.tera", &context)?;
                let minified = minify(rendered.as_bytes(), &minify_cfg);
                safely_write_file(&output_path, String::from_utf8(minified)?.as_str())?;

                println!(
                    "{} {} -> {} (with and lazy loading)",
                    "Converting".green(),
                    entry
                        .path()
                        .display()
                        .to_string()
                        .replace('\\', "/")
                        .yellow(),
                    output_path
                        .display()
                        .to_string()
                        .replace('\\', "/")
                        .yellow(),
                );
            } else {
                process_content_images(&entry, &dist_static, &lazy_dir, &config)?;
            }
        } else if entry.path().is_dir() && entry.path().display().to_string() != "content" {
            let file_name = entry.file_name().to_string_lossy();
            if file_name.starts_with(".") {
                continue;
            }

            let relative_path = entry
                .path()
                .strip_prefix("content")?
                .to_string_lossy()
                .replace('\\', "/");
            let output_dir = dist.join(relative_path.replace('/', "\\"));
            create_directory_safely(&output_dir)?;
            let items = create_listing(entry.path())?;

            let mut context = tera::Context::new();
            context.insert("items", &items);
            context.insert("dir_path", &relative_path);
            context.insert("compress_to_webp", &config.images.compress_to_webp);
            let rendered = tera.render("listing.tera", &context)?;
            let minified = minify(rendered.as_bytes(), &minify_cfg);
            safely_write_file(
                &output_dir.join("index.html"),
                String::from_utf8(minified)?.as_str(),
            )?;

            println!(
                "{} {} -> {}",
                "Creating listing for".green(),
                entry
                    .path()
                    .display()
                    .to_string()
                    .replace('\\', "/")
                    .yellow(),
                output_dir.display().to_string().replace('\\', "/").yellow()
            );
        }
    }

    println!("{}", "Build completed successfully!".green().bold());
    Ok(())
}