use serde_yaml::Value as YamlValue;
use std::error::Error;
use std::path::Path;

use crate::paths::{process_paths, process_wiki_parenthetical_links};

pub fn extract_frontmatter(content: &str) -> Result<(YamlValue, &str), Box<dyn Error>> {
    let trimmed_content = content.trim_start();

    if !trimmed_content.starts_with("---") {
        return Err("Frontmatter is missing".into());
    }

    let end_pattern = "\n---";
    if let Some(end) = trimmed_content[3..].find(end_pattern) {
        let frontmatter_end = 3 + end;
        let frontmatter_str = &trimmed_content[3..frontmatter_end].trim();

        let frontmatter: YamlValue = serde_yaml::from_str(frontmatter_str)?;

        if frontmatter.get("title").is_none() || frontmatter.get("date").is_none() {
            return Err("Missing title or date in frontmatter".into());
        }
        if !frontmatter["title"].is_string() || !frontmatter["date"].is_string() {
            return Err("Title and date must be strings".into());
        }

        let md_content = &trimmed_content[frontmatter_end + end_pattern.len()..];
        Ok((frontmatter, md_content))
    } else {
        Err("Frontmatter end delimiter not found".into())
    }
}

pub fn markdown_to_html(markdown: &str, file_path: &Path) -> String {
    let mut processed_markdown = process_paths(markdown, file_path);
    processed_markdown = process_wiki_parenthetical_links(&processed_markdown);

    let mut html = String::new();
    let options = pulldown_cmark::Options::ENABLE_GFM | pulldown_cmark::Options::ENABLE_STRIKETHROUGH;
    let parser = pulldown_cmark::Parser::new_ext(&processed_markdown, options);
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}