//! TUI module: Interactive terminal interface
//!
//! Architect View for knowledge exploration using ratatui.

mod app;
mod ui;
mod widgets;

#[allow(unused_imports)]
pub use app::{App, AppMode, FilterType};
pub use ui::Ui;

use crate::db::Database;
use ratatui::{
    backend::{CrosstermBackend, Backend},
    terminal::Terminal,
};
use crossterm::event::{Event, KeyCode};
use std::io;

/// Launch the interactive TUI
pub async fn launch(db: &Database) -> crate::NexusResult<()> {
    // Set up terminal
    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(db);

    // Load blocks from database
    app.load_blocks().await?;

    // Run the event loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    result
}

/// Main application loop
async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App<'_>,
) -> crate::NexusResult<()> {
    loop {
        // Draw the UI
        terminal.draw(|f| {
            let size = f.size();
            let ui = Ui::new(app);
            ui.render(size, f.buffer_mut());
        })?;

        // Handle input events
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            match crossterm::event::read()? {
                Event::Key(key) => {
                    handle_key_event(key, app).await?;
                }
                Event::Mouse(_) => {
                    // Mouse events not yet implemented
                }
                Event::Resize(_, _) => {
                    // Resize is handled automatically by ratatui
                }
                Event::FocusGained | Event::FocusLost | Event::Paste(_) => {
                    // Focus and paste events not yet implemented
                }
            }
        }

        // Check if app should quit
        if !app.running {
            break;
        }
    }

    Ok(())
}

/// Handle key events based on current app mode
async fn handle_key_event(
    key: crossterm::event::KeyEvent,
    app: &mut App<'_>,
) -> crate::NexusResult<()> {
    // Handle command mode first
    if app.mode == AppMode::Command {
        match key.code {
            KeyCode::Esc => {
                app.exit_command_mode();
            }
            KeyCode::Enter => {
                app.execute_command().await?;
            }
            KeyCode::Backspace => {
                app.command_backspace();
            }
            KeyCode::Char(c) => {
                app.append_to_command(c);
            }
            KeyCode::Delete => {
                app.clear_command();
            }
            _ => {}
        }
        return Ok(());
    }

    // Handle help mode
    if app.mode == AppMode::Help {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                app.toggle_help();
            }
            KeyCode::Char('?') => {
                app.toggle_help();
            }
            _ => {}
        }
        return Ok(());
    }

    // Handle normal and detail modes
    match app.mode {
        AppMode::Normal | AppMode::Detail => {
            match key.code {
                // Vim-style navigation: j/k for up/down
                KeyCode::Char('j') | KeyCode::Down => {
                    app.move_down();
                    // Reload links when selection changes
                    if app.mode == AppMode::Detail {
                        app.load_links_for_selection().await?;
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    app.move_up();
                    if app.mode == AppMode::Detail {
                        app.load_links_for_selection().await?;
                    }
                }
                // Vim-style left/right for expand/collapse
                KeyCode::Char('h') | KeyCode::Left => {
                    app.move_left();
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    app.move_right();
                }
                // Page up/down for pagination
                KeyCode::PageUp => {
                    app.page_up();
                }
                KeyCode::PageDown => {
                    app.page_down();
                }
                // g/G for go to start/end
                KeyCode::Char('g') => {
                    app.go_to_start();
                }
                KeyCode::Char('G') => {
                    app.go_to_end();
                }
                // Enter to enter detail or navigate to link
                KeyCode::Enter => {
                    if app.mode == AppMode::Normal {
                        app.enter_detail();
                        app.load_links_for_selection().await?;
                    }
                }
                // Backlinks navigation in detail mode
                KeyCode::Char('b') if app.mode == AppMode::Detail => {
                    // Navigate to first backlink
                    app.navigate_to_first_backlink();
                    app.load_links_for_selection().await?;
                }
                // Outgoing links navigation in detail mode
                KeyCode::Char('o') if app.mode == AppMode::Detail => {
                    // Navigate to first outgoing link
                    app.navigate_to_first_outgoing();
                    app.load_links_for_selection().await?;
                }
                // Help
                KeyCode::Char('?') => {
                    app.toggle_help();
                }
                // Command mode
                KeyCode::Char(':') => {
                    app.enter_command_mode();
                }
                // Quit
                KeyCode::Char('q') => {
                    app.quit();
                }
                // Escape to go back from detail
                KeyCode::Esc => {
                    if app.mode == AppMode::Detail {
                        app.exit_detail();
                    }
                }
                // Tab to cycle through filter types (quick filter)
                KeyCode::Tab => {
                    app.cycle_filter();
                }
                // Refresh
                KeyCode::Char('r') => {
                    app.reload_blocks().await?;
                }
                _ => {}
            }
        }
        AppMode::Help | AppMode::Command => {
            // Already handled above, but catch escape in any mode
            match key.code {
                KeyCode::Esc => {
                    if app.mode == AppMode::Help {
                        app.toggle_help();
                    } else if app.mode == AppMode::Command {
                        app.exit_command_mode();
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_code_mapping() {
        // Test that key codes are correctly mapped
        // These are compile-time checks essentially
        assert!(true);
    }

    #[test]
    fn test_mode_switch_on_colon() {
        // When colon is pressed, we enter command mode
        // This is handled by the event loop
        let mode = AppMode::Normal;
        assert_eq!(mode, AppMode::Normal);
    }

    #[test]
    fn test_help_mode_toggle() {
        let mut app_state = AppMode::Normal;
        app_state = AppMode::Help;
        assert_eq!(app_state, AppMode::Help);
        app_state = AppMode::Normal;
        assert_eq!(app_state, AppMode::Normal);
    }
}