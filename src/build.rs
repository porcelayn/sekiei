use crate::config::{Config, ThemeType, get_preset_themes};
use crate::{
    file_ops::{clear_directory_safely, create_directory_safely, safely_write_file},
    listing::create_listing,
    markdown::{extract_frontmatter, markdown_to_html},
    utils::is_not_hidden_dir,
};
use css_minify::optimizations::{Level as CssLevel, Minifier as CssMinifier};
use minify_html::minify;
use minify_js::{Session, TopLevelMode, minify as js_minify};
use std::error::Error;
use std::fs;
use std::path::Path;
use tera::Tera;
use walkdir::WalkDir;

pub fn build() -> Result<(), Box<dyn Error>> {
    let dist = Path::new("dist");
    clear_directory_safely(dist)?;
    create_directory_safely(dist)?;
    let dist_static = dist.join("static");
    create_directory_safely(&dist_static)?;

    let config_str = fs::read_to_string("Config.toml")
        .map_err(|e| format!("Failed to read Config.toml: {}", e))?;
    let config: Config =
        toml::from_str(&config_str).map_err(|e| format!("Failed to parse Config.toml: {}", e))?;

    let required_vars = vec![
        "background_color",
        "text_color",
        "link_color",
        "heading_color",
        "code_background",
        "code_text",
        "border_color",
        "accent_color",
        "blockquote_color",
        "secondary_background",
        "secondary_accent",
        "highlight_add",
        "highlight_del",
        "highlight",
        "type",
        "constant",
        "string",
        "comment",
        "keyword",
        "function",
        "variable",
        "punctuation",
        "markup_heading",
        "diff_plus",
        "diff_minus",
        "attribute",
        "constructor",
        "tag",
        "escape",
    ];

    let (light_vars, dark_vars) = match config.theme.theme_type {
        ThemeType::Preset => {
            let preset_name = config
                .theme
                .preset
                .ok_or("Preset name not specified in Config.toml")?;
            let presets = get_preset_themes();
            presets
                .get(&preset_name)
                .ok_or_else(|| format!("Unknown preset theme: {}", preset_name))?
                .clone()
        }
        ThemeType::Custom => {
            let custom = config
                .theme
                .custom
                .ok_or("Custom theme not specified in Config.toml")?;
            (custom.light, custom.dark)
        }
    };

    for var in &required_vars {
        if !light_vars.contains_key(*var) {
            return Err(format!("Missing light theme variable: {}", var).into());
        }
        if !dark_vars.contains_key(*var) {
            return Err(format!("Missing dark theme variable: {}", var).into());
        }
    }

    let mut light_css = String::new();
    for (key, value) in &light_vars {
        let css_key = format!("--{}", key.replace("_", "-"));
        light_css.push_str(&format!("    {}: {};\n", css_key, value));
    }
    let mut dark_css = String::new();
    for (key, value) in &dark_vars {
        let css_key = format!("--{}", key.replace("_", "-"));
        dark_css.push_str(&format!("    {}: {};\n", css_key, value));
    }
    let theme_css = format!(
        r#"
:root {{
{light_css}
}}

@media (prefers-color-scheme: dark) {{
    :root:not([data-theme="light"]) {{
{dark_css}
    }}
}}

[data-theme="light"] {{
{light_css}
}}

[data-theme="dark"] {{
{dark_css}
}}
"#,
        light_css = light_css,
        dark_css = dark_css
    );

    let minified_theme_css = CssMinifier::default()
        .minify(&theme_css, CssLevel::Three)
        .map_err(|e| format!("Failed to minify theme.css: {}", e))?;
    let theme_css_path = dist_static.join("theme.css");
    safely_write_file(&theme_css_path, &minified_theme_css)?;

    println!(
        "Generated and minified theme.css with {} theme",
        config.theme.theme_type.as_str()
    );

    let static_dir = Path::new("static");
    let js_session = Session::new(); // Single session for JS minification
    if static_dir.exists() {
        for entry in WalkDir::new(static_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                let relative_path = entry.path().strip_prefix(static_dir)?;
                let output_path = dist_static.join(relative_path);
                create_directory_safely(output_path.parent().unwrap())?;

                match entry.path().extension().and_then(|s| s.to_str()) {
                    Some("css") => {
                        let css_content = fs::read_to_string(entry.path()).map_err(|e| {
                            format!("Failed to read {}: {}", entry.path().display(), e)
                        })?;
                        let minified_css = CssMinifier::default()
                            .minify(&css_content, CssLevel::Three)
                            .map_err(|e| {
                                format!("Failed to minify {}: {}", entry.path().display(), e)
                            })?;
                        safely_write_file(&output_path, &minified_css)?;
                        println!(
                            "Copying and minifying {} -> {}",
                            entry.path().display(),
                            output_path.display()
                        );
                    }
                    Some("js") => {
                        let js_content = fs::read(entry.path()).map_err(|e| {
                            format!("Failed to read {}: {}", entry.path().display(), e)
                        })?;
                        let mut minified_js = Vec::new();
                        js_minify(
                            &js_session,
                            TopLevelMode::Global,
                            &js_content,
                            &mut minified_js,
                        )
                        .map_err(|e| {
                            format!("Failed to minify {}: {}", entry.path().display(), e)
                        })?;
                        fs::write(&output_path, &minified_js).map_err(|e| {
                            format!("Failed to write minified {}: {}", output_path.display(), e)
                        })?;
                        println!(
                            "Copying and minifying {} -> {}",
                            entry.path().display(),
                            output_path.display()
                        );
                    }
                    _ => {
                        fs::copy(entry.path(), &output_path)?;
                        println!(
                            "Copying {} -> {}",
                            entry.path().display(),
                            output_path.display()
                        );
                    }
                }
            }
        }
    } else {
        println!("No static folder found, skipping static file copy.");
    }

    println!("Loading Templates defined in templates/");
    let tera = Tera::new("templates/**/*").map_err(|e| {
        eprintln!("Error loading templates: {}", e);
        Box::new(e) as Box<dyn Error>
    })?;

    let minify_cfg = minify_html::Cfg {
        minify_js: false,
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
        if entry.path().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("md")
        {
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
                let (html_content, toc) = markdown_to_html(md_content, entry.path());

                let mut context = tera::Context::new();
                let title = frontmatter["title"]
                    .as_str()
                    .unwrap_or("Untitled")
                    .to_string();
                context.insert("title", &title);
                context.insert("markdown", &html_content);
                context.insert("frontmatter", &frontmatter);
                context.insert("table_of_contents", &toc);

                let rendered = tera.render("content.html", &context).map_err(|e| {
                    eprintln!(
                        "Error rendering template for {}: {}",
                        entry.path().display(),
                        e
                    );
                    e
                })?;
                let minified = minify(rendered.as_bytes(), &minify_cfg);
                safely_write_file(&output_path, String::from_utf8(minified).unwrap().as_str())?;

                println!(
                    "Converting {} -> {}",
                    entry.path().display(),
                    output_path.display()
                );
            } else {
                // Restore copying of non-Markdown files (e.g., images) from content to dist/static
                let relative_path = entry.path().strip_prefix("content")?;
                let sanitized_name =
                    crate::utils::sanitize_filename(&relative_path.to_string_lossy());
                let output_path = dist_static.join(&sanitized_name);

                create_directory_safely(output_path.parent().unwrap())?;
                fs::copy(entry.path(), &output_path)?;
                println!(
                    "Copying {} -> {}",
                    entry.path().display(),
                    output_path.display()
                );
            }
        } else if entry.path().is_dir() && entry.path().display().to_string() != "content" {
            let file_name = entry.file_name().to_string_lossy();
            if file_name.starts_with(".") {
                continue;
            }

            let relative_path = entry.path().strip_prefix("content")?;
            let output_dir = dist.join(relative_path);
            create_directory_safely(&output_dir)?;
            let items = create_listing(entry.path())?;

            let mut context = tera::Context::new();
            context.insert("items", &items);
            context.insert("dir_path", &relative_path);

            let rendered = tera.render("listing.html", &context).map_err(|e| {
                eprintln!(
                    "Error rendering template for {}: {}",
                    entry.path().display(),
                    e
                );
                e
            })?;
            let minified = minify(rendered.as_bytes(), &minify_cfg);
            safely_write_file(
                &output_dir.join("index.html"),
                String::from_utf8(minified).unwrap().as_str(),
            )?;

            println!(
                "Creating listing for {} -> {}",
                entry.path().display(),
                output_dir.display()
            );
        }
    }

    println!("Build completed successfully!");
    Ok(())
}
