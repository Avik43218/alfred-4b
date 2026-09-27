use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, InputMode};

pub fn render_input(f: &mut Frame, app: &App, area: Rect) {
    match app.input_mode {
        InputMode::Text => {
            let prompt = Span::styled(" Sir > ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
            let input_text = Span::styled(&app.input_text, Style::default().fg(Color::White));
            let line = Line::from(vec![prompt, input_text]);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(" ⌨️ Text Input (Type prompt and press [Enter]) ")
                .border_style(Style::default().fg(Color::Yellow));

            let paragraph = Paragraph::new(line).block(block);
            f.render_widget(paragraph, area);

            // Set cursor position in text mode
            let cursor_x = area.x + 8 + app.cursor_position as u16;
            let cursor_y = area.y + 1;
            if cursor_x < area.x + area.width - 1 {
                f.set_cursor_position((cursor_x, cursor_y));
            }
        }
        InputMode::Voice => {
            let wave_chars = [" ", "▂", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃", "▂"];
            let mut wave_str = String::new();
            let offset = (app.tick_count % wave_chars.len() as u64) as usize;
            for i in 0..20 {
                let idx = (offset + i) % wave_chars.len();
                wave_str.push_str(wave_chars[idx]);
            }

            let line = if app.is_recording {
                Line::from(vec![
                    Span::styled(" 🔴 [RECORDING ACTIVE] ", Style::default().fg(Color::White).bg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" {} ", wave_str), Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
                    Span::styled("Speak to Alfred... Press [Enter] or [Space] when finished.", Style::default().fg(Color::Yellow)),
                ])
            } else {
                Line::from(vec![
                    Span::styled(" 🎙️ [VOICE STANDBY] ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled("Press ", Style::default().fg(Color::Gray)),
                    Span::styled("[Space] or [Enter]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(" to record query for Whisper STT (GPU).", Style::default().fg(Color::Gray)),
                ])
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .title(" 🎙️ Speech-to-Text Input Mode ")
                .border_style(if app.is_recording { Style::default().fg(Color::Red) } else { Style::default().fg(Color::Cyan) });

            let paragraph = Paragraph::new(line).block(block);
            f.render_widget(paragraph, area);
        }
    }
}
