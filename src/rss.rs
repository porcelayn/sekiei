use crate::{
    config::Config,
    file_ops::safely_write_file,
    lazy_load::add_lazy_loading,
    markdown::{extract_frontmatter, markdown_to_html},
    utils::is_not_hidden_dir,
};
use chrono::{DateTime, Utc, TimeZone};
use rss::{ChannelBuilder, ItemBuilder};
use std::error::Error;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use colored::Colorize;

pub fn generate_rss(dist: &Path, config: &Config) -> Result<(), Box<dyn Error>> {
    println!("{}", "Collecting posts for RSS...".blue());

    let mut posts = Vec::new();
    for entry in WalkDir::new("content")
        .into_iter()
        .filter_entry(is_not_hidden_dir)
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("md")
        {
            let content = fs::read_to_string(entry.path())?;
            let (frontmatter, md_content) = extract_frontmatter(&content)?;
            let relative_path = entry
                .path()
                .strip_prefix("content")?
                .to_string_lossy()
                .replace('\\', "/");
            let url = if relative_path == "index.md" {
                "/".to_string()
            } else {
                format!("/{}", relative_path.replace(".md", ""))
            };

            let date_str = frontmatter["date"]
                .as_str()
                .ok_or("Missing date in frontmatter")?;
            
            let pub_date = parse_custom_date(date_str)
                .map_err(|e| format!("Invalid date format in {}: {}", relative_path, e))?;

            posts.push((
                frontmatter,
                md_content.to_string(),
                url,
                pub_date,
                entry.path().to_path_buf(),
            ));
        }
    }

    posts.sort_by(|a, b| b.3.cmp(&a.3));

    let mut rss_items = Vec::new();
    for (frontmatter, md_content, url, pub_date, path) in posts {
        let title = frontmatter["title"]
            .as_str()
            .unwrap_or("Untitled")
            .to_string();
        let (html_content, _) = markdown_to_html(&md_content, &path);
        let description = Some(add_lazy_loading(&html_content, config.images.compress_to_webp));

        rss_items.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(Some(format!("{}{}", config.general.base_url.clone(),url))) 
                .description(description)
                .pub_date(Some(pub_date.to_rfc2822()))
                .build(),
        );
    }

    let channel = ChannelBuilder::default()
        .title(config.general.title.clone())
        .link(config.general.base_url.clone())
        .description(config.general.description.clone()) 
        .items(rss_items)
        .build();

    let rss_xml = channel.to_string();
    safely_write_file(&dist.join("rss.xml"), &rss_xml)?;
    println!(
        "{} {}",
        "Generated RSS feed at".green(),
        dist.join("rss.xml").display().to_string().yellow()
    );

    Ok(())
}

fn parse_custom_date(date_str: &str) -> Result<DateTime<Utc>, Box<dyn Error>> {
    let formats = ["%d %b %Y", "%d %B %Y", "%Y-%m-%d", "%Y/%m/%d", "%d/%m/%Y"];
    let trimmed_date = date_str.trim();
    
    for format in &formats {
        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(trimmed_date, format) {
            return Ok(Utc.from_utc_datetime(&naive_date.and_hms_opt(0, 0, 0).unwrap()));
        }
    }
    
    Err(format!(
        "Could not parse date '{}'. Expected format '24 Jan 2025' or '24 January 2025'",
        trimmed_date
    ).into())
}