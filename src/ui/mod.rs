pub mod chat;
pub mod header;
pub mod help;
pub mod input;
pub mod modal;
pub mod tools_view;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::App;
use chat::render_chat;
use header::render_header;
use help::render_help;
use input::render_input;
use modal::render_confirmation_modal;
use tools_view::render_tools_view;

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // Vertical layout:
    // [Header] - 3 rows
    // [Main Content: Chat (60%) + Tools (40%)] - Remaining
    // [Input Bar] - 3 rows
    // [Help Bar] - 3 rows
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(8),    // Body (Chat + Tools)
            Constraint::Length(3), // Input
            Constraint::Length(3), // Help footer
        ])
        .split(size);

    // 1. Render Header
    render_header(f, app, main_chunks[0]);

    // 2. Render Main Body (Split horizontally into Chat and Tools Activity)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(62), // Chat Dialogue
            Constraint::Percentage(38), // Tools & Sandboxed Execution
        ])
        .split(main_chunks[1]);

    render_chat(f, app, body_chunks[0]);
    render_tools_view(f, app, body_chunks[1]);

    // 3. Render Input
    render_input(f, app, main_chunks[2]);

    // 4. Render Help Bar
    render_help(f, main_chunks[3]);

    // 5. Render Modal Overlay if pending confirmation
    if let Some(pending) = &app.pending_confirmation {
        render_confirmation_modal(f, pending, size);
    }
}
