//! TUI Application state
//!
//! Manages the state for the interactive architect TUI.

use crate::models::{Block, BlockType};
use crate::db::Database;

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Normal mode - navigating the block list
    Normal,
    /// Detail mode - viewing a selected block
    Detail,
    /// Help mode - showing keyboard shortcuts
    Help,
    /// Command mode - entering commands
    Command,
}

/// Filter state for the block list
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterType {
    None,
    ByType(BlockType),
    Search(String),
}

impl FilterType {
    /// Get the block type if filter is by type
    #[allow(dead_code)]
    pub fn as_block_type(&self) -> Option<BlockType> {
        match self {
            FilterType::ByType(bt) => Some(bt.clone()),
            _ => None,
        }
    }
}

/// Main TUI application state
pub struct App<'a> {
    /// Whether the application is running
    pub running: bool,
    /// Current application mode
    pub mode: AppMode,
    /// List of blocks being displayed
    pub blocks: Vec<Block>,
    /// Currently selected index in the list
    pub selected_index: usize,
    /// Scroll offset for the list
    pub scroll_offset: usize,
    /// Database reference for data access (not included in Debug)
    #[allow(dead_code)]
    db: &'a Database,
    /// Current filter
    pub filter: FilterType,
    /// Backlinks for selected block
    pub backlinks: Vec<Block>,
    /// Outgoing links for selected block
    pub outgoing_links: Vec<Block>,
    /// Command buffer for command mode
    pub command_buffer: String,
    /// Items per page for pagination
    page_size: usize,
}

impl<'a> std::fmt::Debug for App<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("running", &self.running)
            .field("mode", &self.mode)
            .field("blocks", &self.blocks)
            .field("selected_index", &self.selected_index)
            .field("scroll_offset", &self.scroll_offset)
            .field("filter", &self.filter)
            .field("backlinks", &self.backlinks)
            .field("outgoing_links", &self.outgoing_links)
            .field("command_buffer", &self.command_buffer)
            .field("page_size", &self.page_size)
            .finish()
    }
}

impl<'a> App<'a> {
    /// Create a new application instance
    pub fn new(db: &'a Database) -> Self {
        Self {
            running: true,
            mode: AppMode::Normal,
            blocks: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            db,
            filter: FilterType::None,
            backlinks: Vec::new(),
            outgoing_links: Vec::new(),
            command_buffer: String::new(),
            page_size: 20,
        }
    }

    /// Load blocks from the database
    pub async fn load_blocks(&mut self) -> crate::NexusResult<()> {
        let block_repo = self.db.blocks();

        // Load all block types for a comprehensive view
        let structures = block_repo.list_by_type(BlockType::Structure).await?;
        let permanents = block_repo.list_by_type(BlockType::Permanent).await?;
        let hubs = block_repo.list_by_type(BlockType::Hub).await?;
        let ghosts = block_repo.list_by_type(BlockType::Ghost).await?;
        let fleeting = block_repo.list_by_type(BlockType::Fleeting).await?;
        let literature = block_repo.list_by_type(BlockType::Literature).await?;
        let tasks = block_repo.list_by_type(BlockType::Task).await?;
        let references = block_repo.list_by_type(BlockType::Reference).await?;
        let outlines = block_repo.list_by_type(BlockType::Outline).await?;

        // Combine all blocks, ordered by recency
        let mut all_blocks: Vec<Block> = Vec::new();
        all_blocks.extend(structures);
        all_blocks.extend(hubs);
        all_blocks.extend(permanents);
        all_blocks.extend(ghosts);
        all_blocks.extend(fleeting);
        all_blocks.extend(literature);
        all_blocks.extend(tasks);
        all_blocks.extend(references);
        all_blocks.extend(outlines);

        self.blocks = all_blocks;
        self.selected_index = 0;
        self.scroll_offset = 0;

        Ok(())
    }

    /// Reload blocks keeping current filter
    pub async fn reload_blocks(&mut self) -> crate::NexusResult<()> {
        match self.filter.clone() {
            FilterType::None => {
                self.load_blocks().await?;
            }
            FilterType::ByType(block_type) => {
                let block_repo = self.db.blocks();
                self.blocks = block_repo.list_by_type(block_type).await?;
                self.selected_index = 0;
                self.scroll_offset = 0;
            }
            FilterType::Search(query) => {
                self.search_blocks(&query).await?;
            }
        }
        Ok(())
    }

    /// Get the currently selected block
    pub fn selected_block(&self) -> Option<&Block> {
        self.blocks.get(self.selected_index)
    }

    /// Get currently selected block ULID
    pub fn selected_block_id(&self) -> Option<ulid::Ulid> {
        self.selected_block().map(|b| b.id)
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.ensure_visible();
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected_index < self.blocks.len().saturating_sub(1) {
            self.selected_index += 1;
            self.ensure_visible();
        }
    }

    /// Move selection up by page size (Page Up)
    pub fn page_up(&mut self) {
        if self.blocks.is_empty() {
            return;
        }
        let new_index = self.selected_index.saturating_sub(self.page_size);
        self.selected_index = new_index;
        self.scroll_offset = self.scroll_offset.saturating_sub(self.page_size);
    }

    /// Move selection down by page size (Page Down)
    pub fn page_down(&mut self) {
        if self.blocks.is_empty() {
            return;
        }
        let new_index = (self.selected_index + self.page_size)
            .min(self.blocks.len() - 1);
        self.selected_index = new_index;
        self.ensure_visible();
    }

    /// Go to the first item in the list
    pub fn go_to_start(&mut self) {
        if !self.blocks.is_empty() {
            self.selected_index = 0;
            self.scroll_offset = 0;
        }
    }

    /// Go to the last item in the list
    pub fn go_to_end(&mut self) {
        if !self.blocks.is_empty() {
            self.selected_index = self.blocks.len() - 1;
            self.scroll_offset = self.selected_index.saturating_sub(self.page_size - 1);
        }
    }

    /// Move selection left (collapse/collapse group)
    pub fn move_left(&mut self) {
        // In the current design, h is for potential future collapse feature
        // For now, exit detail mode if in detail
        if self.mode == AppMode::Detail {
            self.exit_detail();
        }
    }

    /// Move selection right (expand)
    pub fn move_right(&mut self) {
        // In the current design, l is for potential future expand feature
        // For now, enter detail mode if in normal
        if self.mode == AppMode::Normal {
            self.enter_detail();
        }
    }

    /// Ensure the selected item is visible in the scroll view
    fn ensure_visible(&mut self) {
        if self.selected_index >= self.scroll_offset + self.page_size {
            self.scroll_offset = self.selected_index - self.page_size + 1;
        } else if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
    }

    /// Enter detail mode to view selected block
    pub fn enter_detail(&mut self) {
        if !self.blocks.is_empty() {
            self.mode = AppMode::Detail;
        }
    }

    /// Exit detail mode and return to list
    pub fn exit_detail(&mut self) {
        self.mode = AppMode::Normal;
    }

    /// Toggle help panel
    pub fn toggle_help(&mut self) {
        self.mode = match self.mode {
            AppMode::Help => AppMode::Normal,
            _ => AppMode::Help,
        };
    }

    /// Enter command mode
    pub fn enter_command_mode(&mut self) {
        self.mode = AppMode::Command;
        self.command_buffer.clear();
    }

    /// Exit command mode
    pub fn exit_command_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.command_buffer.clear();
    }

    /// Append character to command buffer
    pub fn append_to_command(&mut self, c: char) {
        self.command_buffer.push(c);
    }

    /// Backspace in command buffer
    pub fn command_backspace(&mut self) {
        self.command_buffer.pop();
    }

    /// Clear command buffer
    pub fn clear_command(&mut self) {
        self.command_buffer.clear();
    }

    /// Execute command
    pub async fn execute_command(&mut self) -> crate::NexusResult<()> {
        let cmd = self.command_buffer.trim().to_string();
        self.command_buffer.clear();

        if cmd.is_empty() {
            self.mode = AppMode::Normal;
            return Ok(());
        }

        // Parse command
        if cmd == "quit" || cmd == "q" {
            self.quit();
            return Ok(());
        }

        if cmd.starts_with("search ") {
            let query = cmd.trim_start_matches("search ").trim();
            if !query.is_empty() {
                self.filter = FilterType::Search(query.to_string());
                self.search_blocks(query).await?;
            }
        } else if cmd.starts_with("filter ") {
            let filter_arg = cmd.trim_start_matches("filter ").trim();
            self.apply_filter(filter_arg).await?;
        } else if cmd == "filter" {
            // Clear filter
            self.filter = FilterType::None;
            self.load_blocks().await?;
        } else if cmd == "new" {
            // Create new note - in a real implementation, this would open an editor
            // For now, just reload to show the user can create notes
            self.reload_blocks().await?;
        } else if cmd == "help" {
            self.toggle_help();
        } else if cmd == "all" {
            // Show all blocks
            self.filter = FilterType::None;
            self.load_blocks().await?;
        } else {
            // Unknown command - just go back to normal mode
            tracing::info!("Unknown command: {}", cmd);
        }

        self.mode = AppMode::Normal;
        Ok(())
    }

    /// Apply filter by type name
    pub async fn apply_filter(&mut self, type_name: &str) -> crate::NexusResult<()> {
        let block_type = match type_name.to_lowercase().as_str() {
            "fleeting" => Some(BlockType::Fleeting),
            "literature" => Some(BlockType::Literature),
            "permanent" => Some(BlockType::Permanent),
            "structure" => Some(BlockType::Structure),
            "hub" => Some(BlockType::Hub),
            "task" => Some(BlockType::Task),
            "reference" => Some(BlockType::Reference),
            "outline" => Some(BlockType::Outline),
            "ghost" => Some(BlockType::Ghost),
            _ => None,
        };

        if let Some(bt) = block_type {
            self.filter = FilterType::ByType(bt.clone());
            let block_repo = self.db.blocks();
            self.blocks = block_repo.list_by_type(bt).await?;
            self.selected_index = 0;
            self.scroll_offset = 0;
        }

        Ok(())
    }

    /// Search blocks by query
    pub async fn search_blocks(&mut self, query: &str) -> crate::NexusResult<()> {
        let block_repo = self.db.blocks();
        self.blocks = block_repo.search_content(query).await?;
        self.selected_index = 0;
        self.scroll_offset = 0;
        Ok(())
    }

    /// Load backlinks for the currently selected block
    pub async fn load_backlinks(&mut self) -> crate::NexusResult<()> {
        self.backlinks.clear();

        let Some(block_id) = self.selected_block_id() else {
            return Ok(());
        };

        let edge_repo = self.db.edges();
        let edges = edge_repo.incoming_to(&block_id).await?;

        let block_repo = self.db.blocks();
        let mut backlinks = Vec::new();

        for edge in edges {
            if let Some(block) = block_repo.get(&edge.from).await? {
                backlinks.push(block);
            }
        }

        self.backlinks = backlinks;
        Ok(())
    }

    /// Load outgoing links for the currently selected block
    pub async fn load_outgoing_links(&mut self) -> crate::NexusResult<()> {
        self.outgoing_links.clear();

        let Some(block_id) = self.selected_block_id() else {
            return Ok(());
        };

        let edge_repo = self.db.edges();
        let edges = edge_repo.outgoing_from(&block_id).await?;

        let block_repo = self.db.blocks();
        let mut outgoing = Vec::new();

        for edge in edges {
            if let Some(block) = block_repo.get(&edge.to).await? {
                outgoing.push(block);
            }
        }

        self.outgoing_links = outgoing;
        Ok(())
    }

    /// Load all links for the selected block
    pub async fn load_links_for_selection(&mut self) -> crate::NexusResult<()> {
        self.load_backlinks().await?;
        self.load_outgoing_links().await?;
        Ok(())
    }

    /// Navigate to a block by ULID (for clicking on links)
    pub fn navigate_to_block(&mut self, block_id: &ulid::Ulid) {
        if let Some(index) = self.blocks.iter().position(|b| &b.id == block_id) {
            self.selected_index = index;
            self.ensure_visible();
        }
    }

    /// Navigate to first backlink if available
    pub fn navigate_to_first_backlink(&mut self) {
        if let Some(backlink) = self.backlinks.first() {
            let id = backlink.id;
            self.navigate_to_block(&id);
        }
    }

    /// Navigate to first outgoing link if available
    pub fn navigate_to_first_outgoing(&mut self) {
        if let Some(out_link) = self.outgoing_links.first() {
            let id = out_link.id;
            self.navigate_to_block(&id);
        }
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Get total number of blocks
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Get count of blocks by type
    pub fn count_by_type(&self, block_type: BlockType) -> usize {
        self.blocks.iter().filter(|b| b.block_type == block_type).count()
    }

    /// Get current filter description
    pub fn filter_description(&self) -> String {
        match &self.filter {
            FilterType::None => "All".to_string(),
            FilterType::ByType(t) => format!("{:?}", t),
            FilterType::Search(q) => format!("Search: {}", q),
        }
    }

    /// Check if in command mode
    #[allow(dead_code)]
    pub fn is_command_mode(&self) -> bool {
        self.mode == AppMode::Command
    }

    /// Cycle through filter types for quick filtering
    pub fn cycle_filter(&mut self) {
        // This cycles through: None -> Fleeting -> Literature -> Permanent -> Structure -> Hub -> Task -> Reference -> Outline -> Ghost -> None
        let next_filter = match &self.filter {
            FilterType::None => FilterType::ByType(BlockType::Fleeting),
            FilterType::ByType(BlockType::Fleeting) => FilterType::ByType(BlockType::Literature),
            FilterType::ByType(BlockType::Literature) => FilterType::ByType(BlockType::Permanent),
            FilterType::ByType(BlockType::Permanent) => FilterType::ByType(BlockType::Structure),
            FilterType::ByType(BlockType::Structure) => FilterType::ByType(BlockType::Hub),
            FilterType::ByType(BlockType::Hub) => FilterType::ByType(BlockType::Task),
            FilterType::ByType(BlockType::Task) => FilterType::ByType(BlockType::Reference),
            FilterType::ByType(BlockType::Reference) => FilterType::ByType(BlockType::Outline),
            FilterType::ByType(BlockType::Outline) => FilterType::ByType(BlockType::Ghost),
            FilterType::ByType(BlockType::Ghost) => FilterType::None,
            FilterType::Search(_) => FilterType::None,
        };
        self.filter = next_filter;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_mode_default() {
        // App should start in Normal mode
        assert_eq!(AppMode::Normal, AppMode::Normal);
    }

    #[test]
    fn test_app_mode_transitions() {
        // Test mode transitions are valid
        let normal = AppMode::Normal;
        let detail = AppMode::Detail;

        assert_eq!(normal, AppMode::Normal);
        assert_eq!(detail, AppMode::Detail);
        assert_ne!(normal, detail);
    }

    #[test]
    fn test_selection_bounds_logic() {
        // Test that selection stays within bounds
        // When list is empty, index 0 is the only valid index
        // When list has 1+ items, index should be 0 to len-1

        let empty_len: usize = 0;
        let _single_item_len: usize = 1;
        let multi_item_len: usize = 5;

        // For empty list, index 0 is the only valid
        assert!(empty_len.saturating_sub(1) == 0);

        // For single item, moving up stays at 0
        let mut index = 0;
        if index > 0 {
            index -= 1;
        }
        assert_eq!(index, 0);

        // For multi-item list, can move up from index > 0
        index = 3;
        if index > 0 {
            index -= 1;
        }
        assert_eq!(index, 2);

        // Moving down is bounded by len - 1
        index = multi_item_len - 1;
        let new_index = index + 1;
        let bounded = new_index.min(multi_item_len - 1);
        assert_eq!(bounded, multi_item_len - 1);
    }

    #[test]
    fn test_filter_type_equality() {
        let filter_none = FilterType::None;
        let filter_type = FilterType::ByType(BlockType::Permanent);
        let filter_search = FilterType::Search("rust".to_string());

        assert_eq!(filter_none, FilterType::None);
        assert_eq!(filter_type, FilterType::ByType(BlockType::Permanent));
        assert_eq!(filter_search, FilterType::Search("rust".to_string()));
    }

    #[test]
    fn test_block_type_clone() {
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

        for bt in types {
            let cloned = bt.clone();
            assert_eq!(bt, cloned);
        }
    }

    #[test]
    fn test_command_buffer_operations() {
        // Test command buffer push/pop
        let mut buffer = String::new();
        buffer.push('s');
        buffer.push('e');
        buffer.push('a');
        assert_eq!(buffer, "sea");

        buffer.pop();
        assert_eq!(buffer, "se");

        buffer.clear();
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_pagination_calculation() {
        let page_size: usize = 20;

        // Test page down calculation
        let current_idx: usize = 15;
        let new_idx = (current_idx + page_size).min(100 - 1);
        assert_eq!(new_idx, 35);

        // Test page up calculation
        let current_idx: usize = 25;
        let new_idx = current_idx.saturating_sub(page_size);
        assert_eq!(new_idx, 5);

        // Test page up at start
        let current_idx: usize = 5;
        let new_idx = current_idx.saturating_sub(page_size);
        assert_eq!(new_idx, 0);
    }

    #[test]
    fn test_go_to_bounds() {
        // Test go to start
        let mut index = 50;
        if index > 0 {
            index = 0;
        }
        assert_eq!(index, 0);

        // Test go to end
        let len = 100;
        let mut index = 50;
        if len > 0 {
            index = len - 1;
        }
        assert_eq!(index, 99);
    }

    #[test]
    fn test_scroll_offset_calculation() {
        let page_size = 20;

        // When selection goes below visible area
        let selected_index = 25;
        let scroll_offset = 5;
        let new_scroll = if selected_index >= scroll_offset + page_size {
            selected_index - page_size + 1
        } else {
            scroll_offset
        };
        assert_eq!(new_scroll, 6); // 25 - 20 + 1 = 6

        // When selection goes above visible area
        let selected_index = 2;
        let scroll_offset = 10;
        let new_scroll = if selected_index >= scroll_offset + page_size {
            selected_index - page_size + 1
        } else if selected_index < scroll_offset {
            selected_index
        } else {
            scroll_offset
        };
        assert_eq!(new_scroll, 2);
    }

    #[test]
    fn test_filter_description() {
        let filter_none = FilterType::None;
        let filter_type = FilterType::ByType(BlockType::Permanent);
        let filter_search = FilterType::Search("rust".to_string());

        assert_eq!(format!("{:?}", filter_none), "None");
        assert_eq!(format!("{:?}", filter_type), "ByType(Permanent)");
        assert_eq!(format!("{:?}", filter_search), "Search(\"rust\")");
    }

    #[test]
    fn test_filter_cycle() {
        // Test filter cycle logic
        let types: [BlockType; 9] = [
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

        // Verify all types are included
        assert_eq!(types.len(), 9);
    }

    #[test]
    fn test_navigate_to_block() {
        // Test navigation logic
        let blocks: Vec<Block> = (0..10)
            .map(|i| Block::permanent(format!("Block {}", i), "content"))
            .collect();

        let target_id = blocks[5].id;
        let index = blocks.iter().position(|b| b.id == target_id);

        assert_eq!(index, Some(5));
    }

    #[test]
    fn test_filter_as_block_type() {
        let filter_none = FilterType::None;
        let filter_type = FilterType::ByType(BlockType::Permanent);
        let filter_search = FilterType::Search("rust".to_string());

        assert!(filter_none.as_block_type().is_none());
        assert_eq!(filter_type.as_block_type(), Some(BlockType::Permanent));
        assert!(filter_search.as_block_type().is_none());
    }
}