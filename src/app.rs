use chrono::Local;
use std::collections::VecDeque;

use crate::config::Config;
use crate::ollama::types::ChatMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Text,
    Voice,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemStatus {
    Idle,
    Listening,
    Transcribing,
    Thinking,
    Searching(String),
    Executing(String),
    AwaitingConfirmation,
    Speaking,
    Error(String),
}

impl SystemStatus {
    pub fn label(&self) -> String {
        match self {
            SystemStatus::Idle => "IDLE".to_string(),
            SystemStatus::Listening => "LISTENING (Mic Active)".to_string(),
            SystemStatus::Transcribing => "TRANSCRIBING (Whisper GPU)".to_string(),
            SystemStatus::Thinking => "THINKING (Ollama CPU/RAM)".to_string(),
            SystemStatus::Searching(q) => format!("SEARCHING: {}", q),
            SystemStatus::Executing(c) => format!("EXECUTING: {}", c),
            SystemStatus::AwaitingConfirmation => "CONFIRMATION REQUIRED".to_string(),
            SystemStatus::Speaking => "SPEAKING (XTTS GPU)".to_string(),
            SystemStatus::Error(e) => format!("ERROR: {}", e),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Alfred,
    System,
    Tool,
}

#[derive(Debug, Clone)]
pub struct DisplayMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: String,
    pub is_voice: bool,
}

#[derive(Debug, Clone)]
pub enum ToolLogKind {
    Search {
        query: String,
        results_count: usize,
        summary: String,
    },
    Command {
        command: String,
        exit_code: Option<i32>,
        snippet: String,
        duration_ms: u128,
        allowed: bool,
    },
}

#[derive(Debug, Clone)]
pub struct ToolLogEntry {
    pub timestamp: String,
    pub kind: ToolLogKind,
}

#[derive(Debug, Clone)]
pub struct PendingConfirmation {
    pub command: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct DeviceStats {
    pub ollama_connected: bool,
    pub voice_connected: bool,
    pub whisper_loaded: bool,
    pub xtts_loaded: bool,
    pub vram_used_mb: f32,
    pub is_mock_voice: bool,
}

pub struct App {
    pub config: Config,
    pub input_mode: InputMode,
    pub status: SystemStatus,
    pub input_text: String,
    pub cursor_position: usize,
    pub messages: Vec<DisplayMessage>,
    pub chat_history: Vec<ChatMessage>,
    pub tool_logs: VecDeque<ToolLogEntry>,
    pub pending_confirmation: Option<PendingConfirmation>,
    pub device_stats: DeviceStats,
    pub is_recording: bool,
    pub tick_count: u64,
    #[allow(dead_code)]
    pub scroll_offset: usize,
    pub should_quit: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        let mut app = Self {
            config,
            input_mode: InputMode::Text,
            status: SystemStatus::Idle,
            input_text: String::new(),
            cursor_position: 0,
            messages: Vec::new(),
            chat_history: Vec::new(),
            tool_logs: VecDeque::with_capacity(50),
            pending_confirmation: None,
            device_stats: DeviceStats::default(),
            is_recording: false,
            tick_count: 0,
            scroll_offset: 0,
            should_quit: false,
        };

        // Welcome greeting from Alfred
        let now = Local::now().format("%H:%M:%S").to_string();
        let greeting = if app.config.ollama.num_gpu > 0 {
            format!(
                "Good evening, Sir. A.L.F.R.E.D. is initialized and at your service. Ollama is operating with {} layers accelerated on GPU VRAM, and Coqui XTTS is ready on CUDA. How may I assist you today?",
                app.config.ollama.num_gpu
            )
        } else {
            "Good evening, Sir. A.L.F.R.E.D. is initialized and at your service. Ollama is allocated strictly to CPU system RAM, and GPU VRAM is reserved for Whisper and Coqui XTTS. How may I assist you today?".into()
        };
        app.messages.push(DisplayMessage {
            role: MessageRole::Alfred,
            content: greeting,
            timestamp: now,
            is_voice: false,
        });

        // Initialize Ollama system prompt with Butler persona
        app.chat_history.push(ChatMessage {
            role: "system".into(),
            content: r#"You are A.L.F.R.E.D., a hyper-competent, highly advanced AI digital butler.
The user is your creator and Boss. Address him strictly as 'Sir'.
Speak with elegant British formality, casual politeness, and razor-sharp dry wit.
Keep responses concise, sophisticated, and short (3-4 sentences maximum).
CRITICAL DIRECTIVE FOR TOOLS & SEARCH:
- If asked to run a command or open a program: Output [EXECUTE: <command>]
- If asked to search the web or fetch live information: Output [SEARCH: <topic>]
- If asked to make tea: Output [EXECUTE: brew-tea]
- When provided with [SEARCH_RESULTS: ...] or [COMMAND_OUTPUT: ...], the tool execution is ALREADY COMPLETE.
  Do NOT output [SEARCH: ...] or [EXECUTE: ...] again. Answer Sir's question directly using the provided data."#.into(),
            tool_calls: None,
        });

        app
    }

    pub fn toggle_input_mode(&mut self) {
        self.input_mode = match self.input_mode {
            InputMode::Text => InputMode::Voice,
            InputMode::Voice => InputMode::Text,
        };
    }

    pub fn add_user_message(&mut self, text: String, is_voice: bool) {
        self.scroll_offset = 0; // Reset scroll position so new conversation is visible
        let now = Local::now().format("%H:%M:%S").to_string();
        self.messages.push(DisplayMessage {
            role: MessageRole::User,
            content: text.clone(),
            timestamp: now,
            is_voice,
        });
        self.chat_history.push(ChatMessage {
            role: "user".into(),
            content: text,
            tool_calls: None,
        });
    }

    pub fn add_assistant_message(&mut self, text: String, is_voice: bool) {
        let now = Local::now().format("%H:%M:%S").to_string();
        self.messages.push(DisplayMessage {
            role: MessageRole::Alfred,
            content: text.clone(),
            timestamp: now,
            is_voice,
        });
        self.chat_history.push(ChatMessage {
            role: "assistant".into(),
            content: text,
            tool_calls: None,
        });
    }

    pub fn add_system_message(&mut self, text: String) {
        let now = Local::now().format("%H:%M:%S").to_string();
        self.messages.push(DisplayMessage {
            role: MessageRole::System,
            content: text,
            timestamp: now,
            is_voice: false,
        });
    }

    pub fn add_tool_message(&mut self, text: String) {
        let now = Local::now().format("%H:%M:%S").to_string();
        self.messages.push(DisplayMessage {
            role: MessageRole::Tool,
            content: text,
            timestamp: now,
            is_voice: false,
        });
    }

    pub fn log_search(&mut self, query: String, results_count: usize, summary: String) {
        let now = Local::now().format("%H:%M:%S").to_string();
        if self.tool_logs.len() >= 50 {
            self.tool_logs.pop_front();
        }
        self.tool_logs.push_back(ToolLogEntry {
            timestamp: now,
            kind: ToolLogKind::Search {
                query,
                results_count,
                summary,
            },
        });
    }

    pub fn log_command(
        &mut self,
        command: String,
        exit_code: Option<i32>,
        snippet: String,
        duration_ms: u128,
        allowed: bool,
    ) {
        let now = Local::now().format("%H:%M:%S").to_string();
        if self.tool_logs.len() >= 50 {
            self.tool_logs.pop_front();
        }
        self.tool_logs.push_back(ToolLogEntry {
            timestamp: now,
            kind: ToolLogKind::Command {
                command,
                exit_code,
                snippet,
                duration_ms,
                allowed,
            },
        });
    }

    pub fn on_tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(lines);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 100_000;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }
}
