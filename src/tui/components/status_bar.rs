use ratatui::{
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

use crate::tui::state::AppState;

pub struct StatusBar;

impl StatusBar {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        f: &mut Frame,
        area: Rect,
        app_state: &AppState,
        _selection_state: &crate::tui::state::SelectionState,
    ) {
        let status_text = format!(
            " {} items ({} selected) - Space: select, Enter: open dir, c: copy, i: ignores [{}], g: gitignore [{}], b: binary [{}], f: format [{}], n: line numbers [{}], /: search, q: quit ",
            app_state.filtered_items.len(),
            app_state.selected_items.len(),
            if app_state.ignore_config.use_default_ignores { "x" } else { " " },
            if app_state.ignore_config.use_gitignore { "x" } else { " " },
            if app_state.ignore_config.include_binary_files { "x" } else { " " },
            match app_state.output_format {
                crate::models::OutputFormat::Xml => "XML",
                crate::models::OutputFormat::Markdown => "Markdown",
                crate::models::OutputFormat::Json => "JSON",
                crate::models::OutputFormat::Llm => "LLM",
            },
            if app_state.show_line_numbers { "x" } else { " " },
        );

        let block = Block::default().title(status_text).borders(Borders::ALL);
        f.render_widget(block, area);
    }
}
