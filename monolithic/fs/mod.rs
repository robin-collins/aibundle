use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn confirm_overwrite(file_path: &str) -> io::Result<bool> {
    if Path::new(file_path).exists() {
        println!("File '{}' already exists. Overwrite? (y/n): ", file_path);
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        return Ok(input.trim().eq_ignore_ascii_case("y"));
    }
    Ok(true)
}

pub fn list_files(path: &PathBuf) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !is_excluded(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn is_excluded(path: &std::path::Path) -> bool {
    let excluded = ["node_modules", ".git", "target"];
    path.components()
        .any(|c| excluded.contains(&c.as_os_str().to_str().unwrap_or("")))
} 