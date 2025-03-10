use serde::Serialize;
use std::{error::Error, fs, path::Path};

use crate::markdown::extract_frontmatter;

#[derive(Serialize)]
pub struct ListingItem {
    pub name: String,
    pub url: String,
    pub date: String,
    pub description: Option<String>,
}

pub fn create_listing(dir: &Path) -> Result<Vec<ListingItem>, Box<dyn Error>> {
    let mut items = Vec::new();
    for entry in walkdir::WalkDir::new(dir).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        if entry.depth() == 0 { continue; }

        let path = entry.path();
        let name = path.file_name().ok_or("Failed to get file name")?.to_string_lossy().to_string();
        
        if entry.file_type().is_file() && name.ends_with(".md") {
            let rel_path = path.with_extension("").strip_prefix("content")?.to_string_lossy().to_string();
            let url = format!("/{}", rel_path);
            let content = fs::read_to_string(path)?;
            let (frontmatter, _) = extract_frontmatter(&content)?;

            items.push(ListingItem {
                name: frontmatter["title"].as_str().unwrap_or_default().to_string(),
                url,
                date: frontmatter["date"].as_str().unwrap_or_default().to_string(),
                description: frontmatter["description"].as_str().map(|s| s.to_string()),
            });

        } else if entry.file_type().is_file() {
            let rel_path = path.strip_prefix("content")?.to_string_lossy().to_string();
            let sanitized_name = crate::utils::sanitize_filename(&rel_path);
            let url = format!("/static/{}", sanitized_name);
            let metadata = fs::metadata(path)?;
            let modified_time = metadata.modified()?;
            let date = modified_time.duration_since(std::time::UNIX_EPOCH)?.as_secs().to_string();

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