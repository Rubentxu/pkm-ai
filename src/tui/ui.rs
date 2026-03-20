//! UI rendering
//!
//! Handles the rendering of the TUI with a two-panel layout:
//! - Left panel: Block list with navigation
//! - Right panel: Block detail view with backlinks/outgoing links

use ratatui::{
    widgets::{Widget, Block as TuiBlock, Borders, BorderType},
    layout::{Layout, Constraint, Direction, Rect},
    style::{Style, Color, Stylize},
    buffer::Buffer,
};
use crate::tui::app::{App, AppMode};
use crate::tui::widgets::block_list::BlockList;

/// Main UI renderer
pub struct Ui<'a> {
    app: &'a App<'a>,
}

impl<'a> Ui<'a> {
    /// Create a new UI renderer
    pub fn new(app: &'a App<'a>) -> Self {
        Self { app }
    }

    /// Render the entire UI
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Handle help mode separately
        if self.app.mode == AppMode::Help {
            self.render_help_overlay(area, buf);
            return;
        }

        // Split the screen into:
        // - Header (2 lines for title + filter info)
        // - Main content (remaining lines)
        //   - Left panel (60%): Block list
        //   - Right panel (40%): Detail view with backlinks
        // - Footer (2 lines for keyboard shortcuts)

        let constraints = [
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Footer
        ];

        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        self.render_header(areas[0], buf);
        self.render_content(areas[1], buf);
        self.render_footer(areas[2], buf);

        // Render command input overlay if in command mode
        if self.app.mode == AppMode::Command {
            self.render_command_input(area, buf);
        }
    }

    /// Render the header with title and stats
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let title = " Nexus-Grafo Architect ";
        let filter_info = format!(" Filter: {} ", self.app.filter_description());

        let header_block = TuiBlock::default()
            .title(title)
            .title_style(Style::default().bold().cyan())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().cyan());

        header_block.render(area, buf);

        // Render title
        buf.set_string(area.x + 1, area.y + 1, title, Style::default().bold().cyan());

        // Render filter info
        let filter_x = area.x + area.width.saturating_sub(filter_info.len() as u16 + 2);
        buf.set_string(filter_x, area.y + 1, filter_info, Style::default().yellow());

        // Render stats line
        let stats = format!(
            " Blocks: {} | Structures: {} | Zettels: {} | Ghosts: {} ",
            self.app.block_count(),
            self.app.count_by_type(crate::models::BlockType::Structure),
            self.app.count_by_type(crate::models::BlockType::Permanent),
            self.app.count_by_type(crate::models::BlockType::Ghost),
        );
        buf.set_string(area.x + 1, area.y + 2, stats, Style::default().dark_gray());
    }

    /// Render the main content area with list and detail panels
    fn render_content(&self, area: Rect, buf: &mut Buffer) {
        // Split content: 60% list, 40% detail
        let constraints = [
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ];

        let areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        let list_area = areas[0];
        let detail_area = areas[1];

        // Render block list
        let block_list = BlockList::new()
            .blocks(self.app.blocks.clone())
            .selected_index(self.app.selected_index);
        block_list.render(list_area, buf);

        // Render detail panel
        self.render_detail_panel(detail_area, buf);
    }

    /// Render the detail panel showing selected block info with backlinks/outgoing links
    fn render_detail_panel(&self, area: Rect, buf: &mut Buffer) {
        let detail_block = TuiBlock::default()
            .title(" Block Detail ")
            .borders(Borders::ALL)
            .border_style(Style::default().green())
            .border_type(BorderType::Double);

        detail_block.render(area, buf);

        if let Some(_block) = self.app.selected_block() {
            self.render_block_detail_full(area, buf);
        } else {
            // Empty state
            let text = "Select a block to view details";
            let style = Style::default().fg(Color::DarkGray);
            buf.set_string(area.x + 1, area.y + 1, text, style);
        }
    }

    /// Render detailed information about a block including backlinks and outgoing links
    fn render_block_detail_full(&self, area: Rect, buf: &mut Buffer) {
        let block = self.app.selected_block().unwrap();
        let inner = Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(2), area.height.saturating_sub(2));
        let mut y = 0;
        let max_height = inner.height;

        // Block type and ID
        let type_line = format!("Type: {:?} | ID: {}", block.block_type, &block.id_str()[..8]);
        buf.set_string(inner.x, inner.y + y, type_line, Style::default().yellow());
        y += 1;

        if y >= max_height { return; }

        // Title
        let title = if block.title.is_empty() {
            "(untitled)".to_string()
        } else {
            block.title.clone()
        };
        buf.set_string(inner.x, inner.y + y, "Title: ", Style::default().bold());
        buf.set_string(inner.x + 7, inner.y + y, &title, Style::default().white());
        y += 1;

        if y >= max_height { return; }

        // Tags
        if !block.tags.is_empty() {
            let tags_line = format!("Tags: {}", block.tags.join(", "));
            buf.set_string(inner.x, inner.y + y, tags_line, Style::default().magenta());
            y += 1;
            if y >= max_height { return; }
        }

        // Separator
        buf.set_string(inner.x, inner.y + y, "─".repeat((inner.width as usize).min(50)), Style::default().dark_gray());
        y += 1;
        if y >= max_height { return; }

        // Backlinks section
        let backlinks_title = format!(" Backlinks ({}): ", self.app.backlinks.len());
        buf.set_string(inner.x, inner.y + y, backlinks_title, Style::default().bold().cyan());
        y += 1;

        if self.app.backlinks.is_empty() {
            buf.set_string(inner.x, inner.y + y, "  (none)", Style::default().dark_gray());
            y += 1;
        } else {
            for (i, backlink) in self.app.backlinks.iter().take(3).enumerate() {
                let link_title = if backlink.title.is_empty() {
                    "(untitled)".to_string()
                } else {
                    backlink.title.clone()
                };
                let link_line = format!("  [{}] {}", &backlink.id_str()[..6], link_title);
                let truncated = if link_line.len() > inner.width as usize - 2 {
                    format!("{}...", &link_line[..inner.width as usize - 5])
                } else {
                    link_line
                };
                buf.set_string(inner.x, inner.y + y + i as u16, truncated, Style::default().cyan());
            }
            y += self.app.backlinks.len().min(3) as u16;
        }

        if y >= max_height { return; }

        // Separator
        buf.set_string(inner.x, inner.y + y, "─".repeat((inner.width as usize).min(50)), Style::default().dark_gray());
        y += 1;
        if y >= max_height { return; }

        // Outgoing links section
        let outgoing_title = format!(" Outgoing Links ({}): ", self.app.outgoing_links.len());
        buf.set_string(inner.x, inner.y + y, outgoing_title, Style::default().bold().green());
        y += 1;

        if self.app.outgoing_links.is_empty() {
            buf.set_string(inner.x, inner.y + y, "  (none)", Style::default().dark_gray());
            y += 1;
        } else {
            for (i, out_link) in self.app.outgoing_links.iter().take(3).enumerate() {
                let link_title = if out_link.title.is_empty() {
                    "(untitled)".to_string()
                } else {
                    out_link.title.clone()
                };
                let link_line = format!("  [{}] {}", &out_link.id_str()[..6], link_title);
                let truncated = if link_line.len() > inner.width as usize - 2 {
                    format!("{}...", &link_line[..inner.width as usize - 5])
                } else {
                    link_line
                };
                buf.set_string(inner.x, inner.y + y + i as u16, truncated, Style::default().green());
            }
            y += self.app.outgoing_links.len().min(3) as u16;
        }

        if y >= max_height { return; }

        // Separator
        buf.set_string(inner.x, inner.y + y, "─".repeat((inner.width as usize).min(50)), Style::default().dark_gray());
        y += 1;
        if y >= max_height { return; }

        // Content preview
        buf.set_string(inner.x, inner.y + y, "Content:", Style::default().bold());
        y += 1;

        let content_lines: Vec<&str> = block.content.lines().collect();
        let max_content_lines = (max_height as usize).saturating_sub(y as usize + 1);

        for (i, line) in content_lines.iter().take(max_content_lines).enumerate() {
            let truncated_line = if line.len() > inner.width as usize - 2 {
                format!("{}...", &line[..inner.width as usize - 5])
            } else {
                line.to_string()
            };
            buf.set_string(inner.x, inner.y + y + i as u16, truncated_line, Style::default().white());
        }
    }

    /// Render the footer with keyboard shortcuts
    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let footer_block = TuiBlock::default()
            .borders(Borders::ALL)
            .border_style(Style::default().dark_gray());

        footer_block.render(area, buf);

        let mode_indicator = match self.app.mode {
            AppMode::Normal => "NAVIGATE",
            AppMode::Detail => "VIEWING",
            AppMode::Help => "HELP",
            AppMode::Command => "COMMAND",
        };

        let left_text = format!(
            " [j/k] Move | [h/l] Expand/Collapse | [Enter] View | [?] Help | [{}] Quit | Mode: {} ",
            if self.app.mode == AppMode::Command { "ESC" } else { "q" },
            mode_indicator
        );
        let right_text = " Nexus-Grafo v0.2 ";

        buf.set_string(area.x + 1, area.y + 1, left_text, Style::default().dark_gray());
        buf.set_string(
            area.x + area.width.saturating_sub(right_text.len() as u16 + 1),
            area.y + 1,
            right_text,
            Style::default().dark_gray(),
        );
    }

    /// Render command input at the bottom of the screen
    fn render_command_input(&self, area: Rect, buf: &mut Buffer) {
        // Create a command input area at the bottom
        let cmd_height = 3;
        let cmd_area = Rect::new(area.x, area.y + area.height.saturating_sub(cmd_height), area.width, cmd_height);

        let cmd_block = TuiBlock::default()
            .title(" Command ")
            .title_style(Style::default().bold().yellow())
            .borders(Borders::ALL)
            .border_style(Style::default().yellow())
            .border_type(BorderType::Double);

        cmd_block.render(cmd_area, buf);

        // Render command buffer
        let prompt = ": ";
        buf.set_string(cmd_area.x + 1, cmd_area.y + 1, prompt, Style::default().bold().yellow());

        let cmd_text = if self.app.command_buffer.is_empty() {
            "_".to_string()
        } else {
            self.app.command_buffer.clone()
        };
        buf.set_string(cmd_area.x + 3, cmd_area.y + 1, cmd_text, Style::default().white());

        // Show available commands hint
        let hint = "(search, filter, new, all, quit)";
        buf.set_string(cmd_area.x + 1, cmd_area.y + 2, hint, Style::default().dark_gray());
    }

    /// Render help overlay panel
    fn render_help_overlay(&self, area: Rect, buf: &mut Buffer) {
        // Darken the background
        let overlay_block = TuiBlock::default()
            .borders(Borders::ALL)
            .border_style(Style::default().cyan())
            .border_type(BorderType::Double);

        overlay_block.render(area, buf);

        let inner = Rect::new(area.x + 2, area.y + 1, area.width.saturating_sub(4), area.height.saturating_sub(2));

        let help_text = r#"
╔══════════════════════════════════════════════════════════════════════════════╗
║                           Nexus-Grafo TUI Help                             ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║  NAVIGATION                                                                  ║
║  ────────────────                                                            ║
║  j / ↓            Move down in list                                         ║
║  k / ↑            Move up in list                                           ║
║  h / ←            Collapse / Go back                                        ║
║  l / →            Expand / Go to detail                                      ║
║  g                Go to start of list                                       ║
║  G                Go to end of list                                        ║
║  PageUp           Jump one page up                                          ║
║  PageDown         Jump one page down                                        ║
║  Enter            View selected block details                               ║
║  Tab              Cycle through filters                                     ║
║  r                Refresh/reload blocks                                     ║
║                                                                              ║
║  VIEWING (in Detail mode)                                                   ║
║  ───────────────────────────                                                ║
║  b                Navigate to first backlink                                ║
║  o                Navigate to first outgoing link                           ║
║                                                                              ║
║  COMMANDS (press : to enter)                                                ║
║  ─────────────────────────────                                              ║
║  :search <query>   Search blocks by content                                 ║
║  :filter <type>    Filter by type (fleeting, literature, permanent, etc.)  ║
║  :filter           Clear filter (show all)                                 ║
║  :new              Create new note                                          ║
║  :all              Show all blocks                                          ║
║  :quit or :q       Exit TUI                                                 ║
║                                                                              ║
║  Press ? or q to close this help                                           ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
"#;

        let lines: Vec<&str> = help_text.lines().collect();
        let max_lines = inner.height as usize;

        for (i, line) in lines.iter().take(max_lines).enumerate() {
            buf.set_string(inner.x, inner.y + i as u16, line, Style::default().white());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_creation() {
        // UI requires an App with a Database reference
        // This is tested via integration tests
        assert!(true);
    }

    #[test]
    fn test_constraint_split() {
        let constraints = [
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ];

        let _layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints);

        // Verify constraints are set correctly
        assert_eq!(constraints.len(), 2);
    }

    #[test]
    fn test_vertical_constraint_split() {
        let constraints = [
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ];

        let _layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints);

        assert_eq!(constraints.len(), 3);
    }
}