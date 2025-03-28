use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use std::path::{Path, PathBuf};

use crate::models::constants::ICONS;
use crate::tui::state::{AppState, SelectionState};

pub struct FileList;

impl FileList {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState,
        selection_state: &SelectionState,
    ) {
        let display_items = app_state.get_display_items();

        let items: Vec<ListItem> = display_items
            .iter()
            .enumerate()
            .map(|(index, path)| {
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

                let is_selected_line = selection_state.list_state.selected() == Some(index);
                let style = if is_selected_line {
                    Style::default().bg(Color::Gray).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(display_name).style(style)
            })
            .collect();

        let list_widget = List::new(items)
            .block(Block::default().title("Files").borders(Borders::ALL));

        let mut list_state = app_state.list_state.clone();

        f.render_stateful_widget(list_widget, area, &mut list_state);
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
