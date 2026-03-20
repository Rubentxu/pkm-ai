//! CLI Application state

use crate::models::Block;

#[allow(dead_code)]
pub struct App {
    pub running: bool,
    pub current_block: Option<Block>,
}

#[allow(dead_code)]
impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            current_block: None,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
