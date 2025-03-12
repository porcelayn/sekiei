use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::RwLock,
};
use walkdir::WalkDir;

use crate::utils::sanitize_filename;

lazy_static! {
    static ref FILE_CACHE: RwLock<Option<HashMap<String, Vec<PathBuf>>>> = RwLock::new(None);
    static ref IMAGE_REGEX: Regex = Regex::new(r"!\[(.*?)\]\(([^)]+)\)").unwrap();
    static ref ALT_IMAGE_REGEX: Regex = Regex::new(r"!\[\[([^|\]]+)(?:\|([^\]]*))?\]\]").unwrap();
    static ref LINK_REGEX: Regex = Regex::new(r"\[\[([^|\]]+)(?:\|([^\]]*))?\]\]").unwrap();
    static ref WIKI_LINK_REGEX: Regex = Regex::new(r"\[(.*?)\]\(wiki:([^)]+)\)").unwrap();
}

pub fn init_file_cache() {
    let mut cache = FILE_CACHE.write().unwrap();
    if cache.is_none() {
        let mut file_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for entry in WalkDir::new("content").into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let filename = entry.file_name().to_string_lossy().to_string();
                let filename_clone = filename.clone();
                let file_path = entry.path().to_path_buf();

                file_map
                    .entry(filename)
                    .or_insert_with(Vec::new)
                    .push(file_path.clone());

                if filename_clone.ends_with(".md") {
                    if let Some(stem) = entry.path().file_stem() {
                        let stem_str = stem.to_string_lossy().to_string();
                        file_map
                            .entry(stem_str)
                            .or_insert_with(Vec::new)
                            .push(file_path);
                    }
                }
            }
        }

        *cache = Some(file_map);
    }
}

pub fn process_paths(markdown: &str, current_path: &Path) -> String {
    if FILE_CACHE.read().unwrap().is_none() {
        init_file_cache();
    }

    let markdown = process_standard_images(markdown, current_path);
    let markdown = process_alternative_images(&markdown, current_path);
    let markdown = process_links(&markdown);
    let markdown = process_wiki_parenthetical_links(&markdown);
    markdown
}

pub fn process_standard_images(markdown: &str, current_path: &Path) -> String {
    IMAGE_REGEX
        .replace_all(markdown, |caps: &regex::Captures| {
            let alt_text = &caps[1];
            let path = &caps[2];

            if !path.starts_with("http://")
                && !path.starts_with("https://")
                && !path.starts_with('/')
            {
                let static_path = resolve_path(path, current_path);
                format!("![{}]({})", alt_text, static_path)
            } else {
                format!("![{}]({})", alt_text, path)
            }
        })
        .to_string()
}

pub fn process_alternative_images(markdown: &str, current_path: &Path) -> String {
    ALT_IMAGE_REGEX
        .replace_all(markdown, |caps: &regex::Captures| {
            let path = &caps[1];
            let alt_text = caps.get(2).map_or("", |m| m.as_str());

            if !path.starts_with("http://")
                && !path.starts_with("https://")
                && !path.starts_with('/')
            {
                let static_path = find_unique_image(path, current_path);
                format!("![{}]({})", alt_text, static_path)
            } else {
                format!("![{}]({})", alt_text, path)
            }
        })
        .to_string()
}

pub fn process_links(markdown: &str) -> String {
    LINK_REGEX
        .replace_all(markdown, |caps: &regex::Captures| {
            let path = &caps[1];
            let display_text = caps.get(2).map_or_else(
                || {
                    path.strip_prefix("wiki:")
                        .unwrap_or(path)
                        .split('/')
                        .last()
                        .unwrap_or(path)
                },
                |m| m.as_str(),
            );

            if path.starts_with("wiki:") {
                let article = path.strip_prefix("wiki:").unwrap();
                format!(
                    "[wiki:{}](https://en.wikipedia.org/wiki/{})",
                    display_text, article
                )
            } else if !path.starts_with("http://")
                && !path.starts_with("https://")
                && !path.starts_with('/')
            {
                let link_path = if !path.contains('/') {
                    find_unique_internal_link(path)
                } else {
                    get_internal_link_path(path)
                };
                format!("[{}]({})", display_text, link_path)
            } else {
                format!("[{}]({})", display_text, path)
            }
        })
        .to_string()
}

pub fn process_wiki_parenthetical_links(markdown: &str) -> String {
    WIKI_LINK_REGEX
        .replace_all(markdown, |caps: &regex::Captures| {
            let display_text = &caps[1];
            let article = &caps[2];
            format!(
                "[{}](https://en.wikipedia.org/wiki/{})",
                display_text, article
            )
        })
        .to_string()
}

pub fn find_unique_image(image_name: &str, current_path: &Path) -> String {
    if image_name.contains('/') {
        return resolve_path(image_name, current_path);
    }

    let cache = FILE_CACHE.read().unwrap();
    if let Some(file_map) = &*cache {
        if let Some(matches) = file_map.get(image_name) {
            match matches.len() {
                0 => resolve_path(image_name, current_path),
                1 => {
                    let path = &matches[0];
                    format!(
                        "/static/{}",
                        sanitize_filename(
                            &path
                                .strip_prefix("content")
                                .unwrap_or(path)
                                .to_string_lossy()
                        )
                    )
                }
                _ => {
                    let path = &matches[0]; // Just take the first one
                    format!(
                        "/static/{}",
                        sanitize_filename(
                            &path
                                .strip_prefix("content")
                                .unwrap_or(path)
                                .to_string_lossy()
                        )
                    )
                }
            }
        } else {
            resolve_path(image_name, current_path)
        }
    } else {
        resolve_path(image_name, current_path)
    }
}

pub fn find_unique_internal_link(link_name: &str) -> String {
    let cache = FILE_CACHE.read().unwrap();
    if let Some(file_map) = &*cache {
        if let Some(matches) = file_map.get(link_name) {
            match matches.len() {
                0 => get_internal_link_path(link_name),
                _ => {
                    let match_path = matches
                        .iter()
                        .find(|p| p.to_string_lossy().ends_with(".md"))
                        .unwrap_or(&matches[0]);

                    let path = match_path
                        .strip_prefix("content")
                        .unwrap_or(match_path)
                        .with_extension("");
                    let clean_path = path.to_string_lossy().replace('\\', "/");
                    if clean_path == "index" {
                        "/".to_string()
                    } else {
                        format!("/{}", clean_path)
                    }
                }
            }
        } else {
            get_internal_link_path(link_name)
        }
    } else {
        get_internal_link_path(link_name)
    }
}

pub fn get_internal_link_path(path: &str) -> String {
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

pub fn resolve_path(path: &str, current_path: &Path) -> String {
    let current_dir = current_path
        .parent()
        .unwrap_or(Path::new(""))
        .strip_prefix("content")
        .unwrap_or(Path::new(""));

    let relative_path = if path.starts_with("./") || path.starts_with("../") {
        let mut full_path = PathBuf::from(current_dir);
        let path_segments: Vec<&str> = path.split('/').collect();
        let mut path_iter = path_segments.iter();

        match *path_iter.next().unwrap_or(&"") {
            "." => {}
            ".." => {
                if full_path.parent().is_some() {
                    full_path = full_path.parent().unwrap().to_path_buf();
                }
            }
            first_segment => {
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
