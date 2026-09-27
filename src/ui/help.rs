use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_help(f: &mut Frame, area: Rect) {
    let key_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(Color::White);
    let sep_style = Style::default().fg(Color::DarkGray);

    let line = Line::from(vec![
        Span::styled("[Tab]", key_style),
        Span::styled(" Mode  ", desc_style),
        Span::styled("• ", sep_style),
        Span::styled("[Enter]", key_style),
        Span::styled(" Send/Mic  ", desc_style),
        Span::styled("• ", sep_style),
        Span::styled("[↑/↓/PgUp/PgDn]", key_style),
        Span::styled(" Scroll  ", desc_style),
        Span::styled("• ", sep_style),
        Span::styled("[End]", key_style),
        Span::styled(" Bottom  ", desc_style),
        Span::styled("• ", sep_style),
        Span::styled("[Esc]", key_style),
        Span::styled(" Clear  ", desc_style),
        Span::styled("• ", sep_style),
        Span::styled("[Ctrl+C]", key_style),
        Span::styled(" Quit  ", desc_style),
        Span::styled("• ", sep_style),
        Span::styled("Hardware: ", Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
        Span::styled("18L GPU | XTTS CUDA | STT CPU", Style::default().fg(Color::LightCyan)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(line)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}
