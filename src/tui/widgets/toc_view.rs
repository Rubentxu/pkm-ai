//! TOC View Widget

#![allow(dead_code)]

use ratatui::widgets::Widget;
use ratatui::layout::Rect;

/// Table of Contents view widget
pub struct TocView {
    // TODO: Implement properly
}

impl TocView {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for TocView {
    fn render(self, _area: Rect, _buf: &mut ratatui::buffer::Buffer) {
        // TODO: Implement properly
    }
}
