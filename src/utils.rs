pub fn sanitize_filename(path: &str) -> String {
    let p = path.replace("/", "-").replace("\\", "-");
    let mut sanitized = String::new();
    for c in p.chars() {
        if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
            sanitized.push(c);
        } else {
            sanitized.push_str(&format!("-u{:04x}", c as u32));
        }
    }
    sanitized.replace('/', "-")
}

pub fn is_not_hidden_dir(entry: &walkdir::DirEntry) -> bool {
    if entry.file_type().is_dir() {
        entry.file_name()
            .to_str()
            .map_or(false, |name| !name.starts_with('.'))
    } else {
        true
    }
}