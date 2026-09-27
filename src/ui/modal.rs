use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::PendingConfirmation;

pub fn render_confirmation_modal(f: &mut Frame, pending: &PendingConfirmation, area: Rect) {
    // Center popup rectangle
    let popup_width = 70.min(area.width.saturating_sub(4));
    let popup_height = 12.min(area.height.saturating_sub(4));

    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    f.render_widget(Clear, popup_area);

    let lines = vec![
        Line::from(vec![
            Span::styled("⚠️ SECURITY CONFIRMATION REQUIRED", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Alfred requests permission to run shell command:", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(format!("  $ {}", pending.command), Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(format!("Reason: {}", pending.reason), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::White)),
            Span::styled("[Y] Allow Once", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[A] Always Whitelist", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[N/Esc] Deny", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 🛡️ Shell Execution Guardrail ")
        .border_style(Style::default().fg(Color::Red));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, popup_area);
}
