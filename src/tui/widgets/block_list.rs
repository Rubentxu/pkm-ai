//! Block List Widget
//!
//! Displays a navigable list of blocks with selection highlighting.

use ratatui::{
    widgets::{List, ListItem, StatefulWidget, Widget},
    layout::Rect,
    buffer::Buffer,
    style::{Style, Color, Stylize},
};
use crate::models::Block;

/// Block list widget for displaying a list of blocks
pub struct BlockList {
    blocks: Vec<Block>,
    selected_index: usize,
}

impl BlockList {
    /// Create a new BlockList widget
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            selected_index: 0,
        }
    }

    /// Set the blocks to display
    pub fn blocks(mut self, blocks: Vec<Block>) -> Self {
        self.blocks = blocks;
        self
    }

    /// Set the selected index
    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }
}

impl Default for BlockList {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for BlockList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.blocks.is_empty() {
            // Render empty state message
            let empty_text = "No blocks found. Create some blocks first.";
            let style = Style::default().fg(Color::DarkGray);
            for (i, line) in empty_text.lines().enumerate() {
                if i < area.height as usize {
                    buf.set_string(area.x + 1, area.y + i as u16, line, style);
                }
            }
            return;
        }

        // Convert blocks to list items
        let mut items: Vec<ListItem> = Vec::new();
        for block in &self.blocks {
            let block_type_icon = match block.block_type {
                crate::models::BlockType::Fleeting => "F",
                crate::models::BlockType::Literature => "L",
                crate::models::BlockType::Permanent => "P",
                crate::models::BlockType::Structure => "S",
                crate::models::BlockType::Hub => "H",
                crate::models::BlockType::Task => "T",
                crate::models::BlockType::Reference => "R",
                crate::models::BlockType::Outline => "O",
                crate::models::BlockType::Ghost => "G",
            };

            let title = if block.title.is_empty() {
                "(untitled)".to_string()
            } else {
                block.title.clone()
            };

            // Format the line: [T] ULID... | Title
            let ulid_short = &block.id_str()[..8];
            let line = format!("[{}] {} | {}", block_type_icon, ulid_short, title);

            items.push(ListItem::new(line));
        }

        // Create and render the list using StatefulWidget
        let list = List::new(items)
            .block(
                ratatui::widgets::Block::default()
                    .title(" Knowledge Graph ")
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .bold()
            )
            .highlight_symbol(">> ");

        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(self.selected_index));
        StatefulWidget::render(list, area, buf, &mut state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BlockType;

    #[test]
    fn test_block_list_creation() {
        let list = BlockList::new();
        assert_eq!(list.blocks.len(), 0);
        assert_eq!(list.selected_index, 0);
    }

    #[test]
    fn test_block_list_with_blocks() {
        let block = Block::permanent("Test Block", "Test content");
        let list = BlockList::new()
            .blocks(vec![block])
            .selected_index(0);

        assert_eq!(list.blocks.len(), 1);
        assert_eq!(list.selected_index, 0);
    }

    #[test]
    fn test_block_type_icons() {
        // Verify all block types have an icon
        let types = [
            BlockType::Fleeting,
            BlockType::Literature,
            BlockType::Permanent,
            BlockType::Structure,
            BlockType::Hub,
            BlockType::Task,
            BlockType::Reference,
            BlockType::Outline,
            BlockType::Ghost,
        ];

        // This is a compile-time check essentially
        // The icons are used in render(), tested via integration
        assert_eq!(types.len(), 9);
    }
}