use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::utils::sanitize_filename;

pub fn process_paths(markdown: &str, current_path: &Path) -> String {
    let markdown = process_standard_images(markdown, current_path);
    let markdown = process_alternative_images(&markdown, current_path);
    let markdown = process_links(&markdown);
    markdown
}

pub fn process_standard_images(markdown: &str, current_path: &Path) -> String {
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

pub fn process_alternative_images(markdown: &str, current_path: &Path) -> String {
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

pub fn process_links(markdown: &str) -> String {
    let re = Regex::new(r"\[\[([^|\]]+)(?:\|([^\]]*))?\]\]").unwrap();
    
    re.replace_all(markdown, |caps: &regex::Captures| {
        let path = &caps[1];
        let display_text = caps.get(2).map_or_else(
            || path.strip_prefix("wiki:").unwrap_or(path).split('/').last().unwrap_or(path),
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

pub fn process_wiki_parenthetical_links(markdown: &str) -> String {
    let re = Regex::new(r"\[(.*?)\]\(wiki:([^)]+)\)").unwrap();
    
    re.replace_all(markdown, |caps: &regex::Captures| {
        let display_text = &caps[1];
        let article = &caps[2];
        format!("[{}](https://en.wikipedia.org/wiki/{})", display_text, article)
    }).to_string()
}

pub fn find_unique_image(image_name: &str, current_path: &Path) -> String {
    if image_name.contains('/') {
        return resolve_path(image_name, current_path);
    }
    
    let mut matches = Vec::new();
    
    for entry in WalkDir::new("content").into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() { continue; }
        if entry.file_name().to_string_lossy() == image_name {
            matches.push(entry.path().to_path_buf());
        }
    }
    
    match matches.len() {
        0 => resolve_path(image_name, current_path),
        1 => format!("/static/{}", sanitize_filename(&matches[0].strip_prefix("content").unwrap_or(&matches[0]).to_string_lossy())),
        _ => {
            for entry in WalkDir::new("content").into_iter().filter_map(|e| e.ok()) {
                if !entry.file_type().is_file() { continue; }
                if entry.file_name().to_string_lossy() == image_name {
                    let match_path = entry.path().strip_prefix("content").unwrap_or(entry.path());
                    return format!("/static/{}", sanitize_filename(&match_path.to_string_lossy()));
                }
            }
            resolve_path(image_name, current_path)
        }
    }
}

pub fn find_unique_internal_link(link_name: &str) -> String {
    let mut matches = Vec::new();
    
    for entry in WalkDir::new("content").into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() { continue; }
        let file_name = entry.file_name().to_string_lossy();
        let file_stem = entry.path().file_stem().unwrap_or_default().to_string_lossy();
        
        if file_name.ends_with(".md") && file_stem == link_name {
            matches.push(entry.path().to_path_buf());
        }
    }
    
    match matches.len() {
        0 => get_internal_link_path(link_name),
        1 => {
            let match_path = matches[0].strip_prefix("content").unwrap_or(&matches[0]).with_extension("");
            let clean_path = match_path.to_string_lossy().replace('\\', "/");
            if clean_path == "index" { "/".to_string() } else { format!("/{}", clean_path) }
        }
        _ => {
            for entry in WalkDir::new("content").into_iter().filter_map(|e| e.ok()) {
                if !entry.file_type().is_file() { continue; }
                let file_stem = entry.path().file_name().unwrap_or_default().to_string_lossy();
                if file_stem.ends_with(".md") && file_stem == link_name {
                    let match_path = entry.path().strip_prefix("content").unwrap_or(entry.path()).with_extension("");
                    return format!("/{}", match_path.to_string_lossy().replace('\\', "/"));
                }
            }
            get_internal_link_path(link_name)
        }
    }
}

pub fn get_internal_link_path(path: &str) -> String {
    let clean_path = if path.ends_with(".md") { &path[0..path.len() - 3] } else { path };
    if clean_path == "index" { "/".to_string() } else { format!("/{}", clean_path) }
}

pub fn resolve_path(path: &str, current_path: &Path) -> String {
    let current_dir = current_path.parent().unwrap().strip_prefix("content").unwrap_or(Path::new(""));
    
    let relative_path = if path.starts_with("./") || path.starts_with("../") {
        let mut full_path = PathBuf::from(current_dir);
        let path_segments: Vec<&str> = path.split('/').collect();
        let mut path_iter = path_segments.iter();
        
        match *path_iter.next().unwrap_or(&"") {
            "." => {},
            ".." => { if full_path.parent().is_some() { full_path = full_path.parent().unwrap().to_path_buf(); } },
            first_segment => { full_path.push(first_segment); }
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