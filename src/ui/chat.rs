use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, MessageRole};

pub fn render_chat(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    for msg in &app.messages {
        let (role_prefix, role_style, content_style) = match msg.role {
            MessageRole::User => {
                let prefix = if msg.is_voice { " 🎤 Boss (Voice)" } else { " 👤 Boss" };
                (
                    prefix,
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    Style::default().fg(Color::White),
                )
            }
            MessageRole::Alfred => {
                let prefix = if msg.is_voice { " 🎩 Alfred (Spoken)" } else { " 🎩 Alfred" };
                (
                    prefix,
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    Style::default().fg(Color::Rgb(240, 230, 200)),
                )
            }
            MessageRole::System => (
                " ⚙️ System",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::Gray),
            ),
            MessageRole::Tool => (
                " 🛠️ Tool Activity",
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::LightMagenta),
            ),
        };

        // Header line: [Role] (Timestamp)
        lines.push(Line::from(vec![
            Span::styled(role_prefix, role_style),
            Span::styled(format!(" [{}]", msg.timestamp), Style::default().fg(Color::DarkGray)),
        ]));

        // Content lines (supports multi-line text)
        for sub_line in msg.content.lines() {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(sub_line.to_string(), content_style),
            ]));
        }

        // Empty separator line
        lines.push(Line::from(""));
    }

    let total_lines = lines.len() as u16;
    let visible_height = area.height.saturating_sub(2);
    let max_scroll = total_lines.saturating_sub(visible_height);
    let scroll_from_bottom = (app.scroll_offset as u16).min(max_scroll);
    let scroll_y = max_scroll.saturating_sub(scroll_from_bottom);

    let title_line = if scroll_from_bottom > 0 {
        Line::from(vec![
            Span::styled(" 💬 Butler Dialogue ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!(" [↑ Scrolled Up {} lines | Press 'End' or '↓' for Latest] ", scroll_from_bottom),
                Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(" 💬 Butler Dialogue ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" [Latest] ", Style::default().fg(Color::DarkGray)),
        ])
    };

    let border_color = if scroll_from_bottom > 0 {
        Color::Yellow
    } else {
        Color::Cyan
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title_line)
        .border_style(Style::default().fg(border_color));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll_y, 0));

    f.render_widget(paragraph, area);

    // Render interactive scrollbar on the right border
    if max_scroll > 0 {
        let scrollbar = ratatui::widgets::Scrollbar::default()
            .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        let mut scrollbar_state =
            ratatui::widgets::ScrollbarState::new(max_scroll as usize).position(scroll_y as usize);
        f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}
