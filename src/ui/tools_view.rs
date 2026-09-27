use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, ToolLogKind};

pub fn render_tools_view(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    if app.tool_logs.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(" No active tool calls.", Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(" Alfred will log web searches and shell executions here.", Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        for entry in app.tool_logs.iter().rev() {
            match &entry.kind {
                ToolLogKind::Search {
                    query,
                    results_count,
                    summary,
                } => {
                    lines.push(Line::from(vec![
                        Span::styled(" 🌐 [SEARCH] ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("\"{}\"", query), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" [{}]", entry.timestamp), Style::default().fg(Color::DarkGray)),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled(format!("    Found {} sources: {}", results_count, summary), Style::default().fg(Color::LightBlue)),
                    ]));
                    lines.push(Line::from(""));
                }
                ToolLogKind::Command {
                    command,
                    exit_code,
                    snippet,
                    duration_ms,
                    allowed,
                } => {
                    let (status_badge, badge_style) = if !allowed {
                        (" [BLOCKED] ", Style::default().fg(Color::White).bg(Color::Red))
                    } else if *exit_code == Some(0) {
                        (" [EXEC OK] ", Style::default().fg(Color::Black).bg(Color::Green))
                    } else {
                        (" [EXEC ERR] ", Style::default().fg(Color::White).bg(Color::Red))
                    };

                    lines.push(Line::from(vec![
                        Span::styled(status_badge, badge_style.add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" $ {}", command), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" ({}ms) [{}]", duration_ms, entry.timestamp), Style::default().fg(Color::DarkGray)),
                    ]));

                    if !snippet.is_empty() {
                        for sub in snippet.lines().take(4) {
                            lines.push(Line::from(vec![
                                Span::raw("    "),
                                Span::styled(sub, Style::default().fg(Color::Gray)),
                            ]));
                        }
                    }
                    lines.push(Line::from(""));
                }
            }
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 🛠️ Tool Activity & Sandbox ")
        .border_style(Style::default().fg(Color::Magenta));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}
