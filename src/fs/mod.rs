use std::path::PathBuf;
use walkdir::WalkDir;

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