use crate::paths::{process_paths, process_wiki_parenthetical_links};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};
use serde::Serialize;
use serde_yaml::Value as YamlValue;
use std::error::Error;
use std::path::Path;

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

#[derive(Debug, Serialize)]
pub struct TOCEntry {
    level: u32,
    title: String,
    id: String,
}

pub fn markdown_to_html(markdown: &str, file_path: &Path) -> (String, Vec<TOCEntry>) {
    let mut processed_markdown = process_paths(markdown, file_path);
    processed_markdown = process_wiki_parenthetical_links(&processed_markdown);

    let mut html = String::new();
    let mut toc = Vec::new();
    let options = Options::ENABLE_GFM
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_MATH
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_DEFINITION_LIST
        | Options::ENABLE_SMART_PUNCTUATION;

    let parser = Parser::new_ext(&processed_markdown, options);

    let mut events = Vec::new();
    let mut current_heading: Option<(u32, Vec<Event>)> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_heading = Some((level as u32, Vec::new()));
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((level, inner_events)) = current_heading.take() {
                    let mut text_content = String::new();
                    for e in &inner_events {
                        if let Event::Text(t) = e {
                            text_content.push_str(t);
                        }
                    }
                    let slug = text_content
                        .trim()
                        .to_lowercase()
                        .replace(' ', "-")
                        .replace(|c: char| !c.is_alphanumeric() && c != '-', "");
                    toc.push(TOCEntry {
                        level,
                        id: slug.clone(),
                        title: text_content.clone(),
                    });

                    let mut inner_html = String::new();
                    html::push_html(&mut inner_html, inner_events.into_iter());
                    let heading_html =
                        format!("<h{} id=\"{}\">{}</h{}>", level, slug, inner_html, level);
                    events.push(Event::Html(heading_html.into()));
                }
            }
            _ => {
                if let Some((_, ref mut inner_events)) = current_heading {
                    inner_events.push(event);
                } else {
                    events.push(event);
                }
            }
        }
    }

    html::push_html(&mut html, events.into_iter());
    (html, toc)
}