use std::time::Instant;

pub struct Modal {
    pub message: String,
    pub timestamp: Instant,
    pub width: u16,
    pub height: u16,
    pub page: usize,
}

impl Modal {
    pub fn new(message: String, width: u16, height: u16) -> Self {
        Self {
            message,
            timestamp: Instant::now(),
            width,
            height,
            page: 0,
        }
    }

    pub fn copy_stats(
        file_count: usize,
        folder_count: usize,
        line_count: usize,
        byte_size: usize,
        format: &crate::models::OutputFormat,
    ) -> Self {
        Self::new(
            format!(
                "Files copied: {}\n\
                 Folders copied: {}\n\
                 Total lines: {}\n\
                 Total size: {}\n\
                 Format: {}\n",
                file_count,
                folder_count,
                line_count,
                crate::utils::human_readable_size(byte_size),
                match format {
                    crate::models::OutputFormat::Xml => "XML",
                    crate::models::OutputFormat::Markdown => "Markdown",
                    crate::models::OutputFormat::Json => "JSON",
                    crate::models::OutputFormat::Llm => "LLM",
                }
            ),
            45,
            8,
        )
    }

    pub fn help() -> Self {
        let help_text = "Keyboard Shortcuts\n\
═════════════════\n\
\n\
Navigation\n\
──────────\n\
↑/↓        - Move selection\n\
PgUp/PgDn  - Move by 10 items\n\
Enter      - Open directory\n\
Tab        - Expand/collapse folder\n\
\n\
Selection\n\
─────────\n\
Space      - Select/deselect item\n\
*          - Select/deselect all\n\
\n\
Actions\n\
───────\n\
c          - Copy to clipboard\n\
f          - Toggle format (XML/MD/JSON)\n\
n          - Toggle line numbers\n\
/          - Search (ESC to cancel)\n\
\n\
Filters\n\
───────\n\
i          - Toggle default ignores\n\
g          - Toggle .gitignore\n\
b          - Toggle binary files\n\
\n\
Other\n\
─────\n\
h          - Show this help\n\
q          - Quit (copies if items selected)\n\
\n\
Help Navigation\n\
──────────────\n\
PgUp/PgDn  - Scroll help pages\n\
Any key    - Close help"
            .to_string();
        Self {
            message: help_text,
            timestamp: Instant::now(),
            width: 60,
            height: 30,
            page: 0,
        }
    }

    pub fn get_visible_content(&self, available_height: u16) -> (String, bool) {
        let content_height = (available_height - 4) as usize;
        let lines: Vec<&str> = self.message.lines().collect();
        let total_lines = lines.len();
        let total_pages = total_lines.div_ceil(content_height);
        let has_more_pages = total_lines > content_height;
        let start = self.page * content_height;
        let end = (start + content_height).min(total_lines);
        let visible_content = lines[start..end].join("\n");
        let content = if has_more_pages {
            format!(
                "{}\n\nPage {} of {}",
                visible_content,
                self.page + 1,
                total_pages
            )
        } else {
            visible_content
        };
        (content, has_more_pages)
    }

    pub fn next_page(&mut self, available_height: u16) {
        let content_height = (available_height - 4) as usize;
        let total_lines = self.message.lines().count();
        let total_pages = total_lines.div_ceil(content_height);
        if total_pages > 1 {
            self.page = (self.page + 1) % total_pages;
        }
    }

    pub fn prev_page(&mut self, available_height: u16) {
        let content_height = (available_height - 4) as usize;
        let total_lines = self.message.lines().count();
        let total_pages = total_lines.div_ceil(content_height);
        if total_pages > 1 {
            self.page = (self.page + total_pages - 1) % total_pages;
        }
    }
}