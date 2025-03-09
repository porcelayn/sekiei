use std::{error::Error, fs, path::Path};

pub fn clear_directory_safely(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    std::fs::create_dir(path)?;
    Ok(())
}

pub fn create_directory_safely(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn safely_write_file(path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
    fs::write(path, content)?;
    Ok(())
}