use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, InputMode, SystemStatus};

pub fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(32), // Title & Butler Crest
            Constraint::Min(40),    // Hardware & Device Placement Badges
            Constraint::Length(28), // Mode & Status Badge
        ])
        .split(area);

    // 1. Title Block
    let title_line = Line::from(vec![
        Span::styled(" 🎩 A.L.F.R.E.D. ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("v1.0 ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Butler AI]", Style::default().fg(Color::Cyan)),
    ]);
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let title_p = Paragraph::new(title_line).block(title_block);
    f.render_widget(title_p, chunks[0]);

    // 2. Hardware Resource & Device Placement Block (Critical 4GB VRAM Display)
    let llm_badge = if app.config.ollama.num_gpu > 0 {
        Span::styled(
            format!(" LLM: GPU ({}L) ", app.config.ollama.num_gpu),
            Style::default().fg(Color::Black).bg(Color::LightGreen).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " LLM: CPU/RAM (0 VRAM) ",
            Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD),
        )
    };
    let stt_badge = if app.config.hardware.stt_device.to_lowercase() == "cpu" {
        Span::styled(
            " STT: CPU (Whisper) ",
            Style::default().fg(Color::Black).bg(Color::Blue).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " STT: GPU (Whisper) ",
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
        )
    };
    let tts_badge = Span::styled(
        " TTS: GPU (XTTS) ",
        Style::default().fg(Color::Black).bg(Color::Magenta).add_modifier(Modifier::BOLD),
    );

    let vram_text = if app.device_stats.vram_used_mb > 0.0 {
        format!(" VRAM: {:.1}/4.0 GB ", app.device_stats.vram_used_mb / 1024.0)
    } else {
        " VRAM Guard: Active ".to_string()
    };
    let vram_badge = Span::styled(
        vram_text,
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    );

    let hw_line = Line::from(vec![
        llm_badge,
        Span::raw(" "),
        stt_badge,
        Span::raw(" "),
        tts_badge,
        Span::raw(" "),
        vram_badge,
    ]);

    let hw_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    let hw_p = Paragraph::new(hw_line).alignment(Alignment::Center).block(hw_block);
    f.render_widget(hw_p, chunks[1]);

    // 3. Current Mode & Live Status
    let mode_badge = match app.input_mode {
        InputMode::Text => Span::styled(
            " [TEXT] ",
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        InputMode::Voice => Span::styled(
            " [VOICE] ",
            Style::default().fg(Color::White).bg(Color::Red).add_modifier(Modifier::BOLD),
        ),
    };

    let status_color = match &app.status {
        SystemStatus::Idle => Color::Green,
        SystemStatus::Listening => Color::LightRed,
        SystemStatus::Transcribing => Color::LightYellow,
        SystemStatus::Thinking => Color::LightCyan,
        SystemStatus::Searching(_) => Color::LightBlue,
        SystemStatus::Executing(_) => Color::LightMagenta,
        SystemStatus::AwaitingConfirmation => Color::Red,
        SystemStatus::Speaking => Color::LightGreen,
        SystemStatus::Error(_) => Color::Red,
    };

    let pulse = if app.tick_count % 2 == 0 { "●" } else { "○" };
    let status_span = Span::styled(
        format!(" {} {}", pulse, app.status.label()),
        Style::default().fg(status_color).add_modifier(Modifier::BOLD),
    );

    let status_line = Line::from(vec![mode_badge, Span::raw(" "), status_span]);
    let status_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(status_color));
    let status_p = Paragraph::new(status_line).alignment(Alignment::Left).block(status_block);
    f.render_widget(status_p, chunks[2]);
}
