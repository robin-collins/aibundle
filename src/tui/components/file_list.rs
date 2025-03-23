use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
};
use std::path::{Path, PathBuf};

use crate::models::ICONS;
use crate::tui::state::AppState;

pub struct FileList {
    pub current_dir: PathBuf,
}

impl FileList {
    pub fn new(current_dir: PathBuf) -> Self {
        Self { current_dir }
    }

    pub fn render<'a>(&self, app_state: &AppState) -> List<'a> {
        let items: Vec<ListItem> = app_state
            .filtered_items
            .iter()
            .map(|path| {
                let depth = path
                    .strip_prefix(&app_state.current_dir)
                    .map(|p| p.components().count())
                    .unwrap_or(0)
                    .saturating_sub(1);
                let indent = "  ".repeat(depth);

                let name = if path.ends_with("..") {
                    "../".to_string()
                } else {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("???")
                        .to_string()
                };

                let icon = Self::get_icon(path);
                let prefix = if app_state.selected_items.contains(path) {
                    "[X] "
                } else {
                    "[ ] "
                };

                let display_name = if path.is_dir() && !path.ends_with("..") {
                    format!("{}{}{} {}/", indent, prefix, icon, name)
                } else {
                    format!("{}{}{} {}", indent, prefix, icon, name)
                };

                ListItem::new(display_name)
            })
            .collect();

        List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Gray))
            .highlight_symbol("> ")
    }

    fn get_icon(path: &Path) -> &'static str {
        if path.is_dir() {
            return ICONS
                .iter()
                .find(|(k, _)| *k == "folder")
                .map(|(_, v)| *v)
                .unwrap_or("📁");
        }

        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| ICONS.iter().find(|(k, _)| *k == ext))
            .map(|(_, v)| *v)
            .unwrap_or(
                ICONS
                    .iter()
                    .find(|(k, _)| *k == "default")
                    .map(|(_, v)| *v)
                    .unwrap_or("📄"),
            )
    }
}
