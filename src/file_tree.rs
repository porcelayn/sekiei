use crate::{
    file_ops::safely_write_file,
    utils::is_not_hidden_dir,
    config::Config,
};
use colored::Colorize;
use std::{
    error::Error,
    path::Path,
    fs::File,
    io::{BufRead, BufReader},
};
use walkdir::WalkDir;
use minify_js::{Session, TopLevelMode, minify as js_minify};
use css_minify::optimizations::{Level as CssLevel, Minifier as CssMinifier};
use serde::{Deserialize, Serialize};

pub fn process_file_tree_assets(dist_static: &Path) -> Result<(), Box<dyn Error>> {
    let js_content = r#"
document.addEventListener('DOMContentLoaded', () => {
    const toggles = document.querySelectorAll('.file-tree .folder-label');
    
    toggles.forEach(toggle => {
        toggle.addEventListener('click', (e) => {
            e.preventDefault();
            const ul = toggle.nextElementSibling;
            if (ul) {
                ul.classList.toggle('hidden');
                const icon = toggle.querySelector('.toggle-icon');
                icon.classList.toggle('rotate-90');
            }
        });
    });
});
"#;
    
    let css_content = r#"
.file-tree ul {
    list-style: none;
    padding-left: 0;
}

.file-tree li {
    margin: 5px 0;
}

.file-tree .directory .folder-contents {
    padding-left: 20px;
}

.file-tree .folder-label {
    cursor: pointer;
}

.file-tree .folder-contents.hidden {
    display: none;
}

.file-tree a {
    text-decoration: none;
}

.file-tree a:hover {
    text-decoration: underline;
}

.toggle-icon {
    display: inline-block;
}
"#;

    let mut minified_js = Vec::new();
    let js_session = Session::new();
    js_minify(
        &js_session,
        TopLevelMode::Global,
        js_content.as_bytes(),
        &mut minified_js,
    ).expect("Failed to minify file_tree.js");
    safely_write_file(&dist_static.join("file_tree.js"), std::str::from_utf8(&minified_js)?)?;
    
    let minified_css = CssMinifier::default()
        .minify(css_content, CssLevel::Three)
        .expect("Failed to minify file_tree.css");
    safely_write_file(&dist_static.join("file_tree.css"), &minified_css)?;

    println!("{}", "Generated and minified file_tree.js and file_tree.css".green());
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

pub fn generate_file_tree_html(config: &Config, current_route: &str) -> Result<String, Box<dyn Error>> {
    let nodes = build_file_tree(Path::new("content"), Path::new(""), config);
    
    let mut html = String::new();
    html.push_str("<div class=\"file-tree\">\n<ul>\n");
    for node in nodes {
        html.push_str(&render_file_node(&node, current_route));
    }
    html.push_str("</ul>\n</div>");
    Ok(html)
}

fn render_file_node(node: &FileNode, current_route: &str) -> String {
    let mut html = String::new();
    let is_current = node.path == current_route || 
                    (current_route == "/" && node.path == "") ||
                    (node.is_dir && current_route.starts_with(&format!("/{}", node.path)));
    
    if node.is_dir {
        let is_expanded = is_current || node.children.iter().any(|child| {
            current_route == child.path || 
            (current_route == "/" && child.path == "") ||
            current_route.starts_with(&format!("/{}", child.path))
        });
        
        html.push_str(&format!(
            "<li class=\"directory mb-1\">\n\
             <div class=\"folder-label flex items-center cursor-pointer text-neutral-600 dark:text-neutral-200 py-1\">\n\
             <span class=\"toggle-icon transform transition-transform duration-200 mr-1 {}\"><i class=\"ph filetree-icon filetree-folder ph-caret-right\"></i></span>\n\
             <span class=\"folder-name text-sm {}\">{}</span>\n\
             </div>\n",
            if is_expanded { "rotate-90" } else { "" },
            if is_current { "font-bold" } else { "" },
            node.name
        ));
        html.push_str(&format!(
            "<ul class=\"folder-contents {} ml-4\">\n",
            if is_expanded { "" } else { "hidden" }
        ));
        for child in &node.children {
            html.push_str(&render_file_node(child, current_route));
        }
        html.push_str("</ul>\n</li>\n");
    } else {
        let icon_class = if node.path.ends_with(".md") {
            "ph filetree-icon filetree-page ph-file-text"
        } else if node.path.ends_with(".webp") || node.path.ends_with(".jpg") || 
                  node.path.ends_with(".jpeg") || node.path.ends_with(".png") {
            "ph ph-image filetree-image"
        } else if node.path.ends_with(".gif") {
            "ph ph-gif filetree-gif"
        } else if node.path.ends_with(".mp4") || node.path.ends_with(".webm") || 
                 node.path.ends_with(".mov") {
            "ph ph-video filetree-video"
        } else {
            "ph ph-file"
        };

        html.push_str(&format!(
            "<li class=\"file mb-1\">\n\
             <a href=\"/{}\" class=\"file-link {}\">\n\
             <i class=\"{} mr-1\"></i>{}\n\
             </a>\n\
             </li>\n",
            node.path,
            if is_current { "font-bold" } else { "" },
            icon_class,
            node.name
        ));
    }
    html
}

pub fn build_file_tree(base: &Path, relative: &Path, config: &Config) -> Vec<FileNode> {
    let full_path = base.join(relative);
    let mut nodes = Vec::new();

    for entry in WalkDir::new(&full_path)
        .min_depth(1)
        .max_depth(1) 
        .into_iter()
        .filter_entry(is_not_hidden_dir)
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if file_name.starts_with('.') {
            continue;
        }

        let is_dir = entry.file_type().is_dir();
        let rel_path = relative.join(&file_name);
        let path_str = rel_path.to_string_lossy().replace('\\', "/");

        if is_dir {
            let children = build_file_tree(base, &rel_path, config);
            nodes.push(FileNode {
                name: file_name,
                path: path_str,
                is_dir,
                children,
            });
        } else {
            let mut name = file_name.clone();
            let mut final_path;

            if path.extension().map_or(false, |ext| ext == "md") {
                let default_name = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    let mut in_frontmatter = false;
                    let mut found_title = false;

                    for line in reader.lines().filter_map(Result::ok) {
                        let trimmed_line = line.trim();
                        if trimmed_line == "---" {
                            if in_frontmatter {
                                break;
                            } else {
                                in_frontmatter = true;
                                continue;
                            }
                        }
                        if in_frontmatter {
                            if let Some((key, value)) = trimmed_line.split_once(':') {
                                let key = key.trim();
                                if key == "title" {
                                    name = value.trim().to_string();
                                    found_title = true;
                                    break;
                                }
                            }
                        }
                    }
                    if !found_title || name.is_empty() {
                        name = default_name.clone();
                    }
                } else {
                    name = default_name.clone();
                }

                final_path = if path_str.ends_with(".md") {
                    let trimmed = &path_str[..path_str.len() - 3];
                    if trimmed == "index" && relative.as_os_str().is_empty() {
                        "".to_string()
                    } else {
                        trimmed.to_string()
                    }
                } else {
                    path_str.clone()
                };
            } else {
                name = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                    
                final_path = format!("static/{}", crate::utils::sanitize_filename(&path_str));
                if config.images.compress_to_webp {
                    if path.extension().map_or(false, |ext| {
                        ext == "jpg" || ext == "jpeg" || ext == "png"
                    }) {
                        final_path = final_path.replace(".jpg", ".webp")
                            .replace(".jpeg", ".webp")
                            .replace(".png", ".webp");
                    }
                }
            }

            nodes.push(FileNode {
                name,
                path: final_path,
                is_dir,
                children: Vec::new(),
            });
        }
    }

    nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    nodes
}