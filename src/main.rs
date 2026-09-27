mod app;
mod config;
mod ollama;
mod tools;
mod ui;
mod voice;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use app::{
    App, DeviceStats, DisplayMessage, InputMode, MessageRole, PendingConfirmation, SystemStatus,
};
use config::Config;
use ollama::{ActionTag, ChatMessage, OllamaClient};
use tools::executor::{CommandExecutor, CommandOutput, GuardrailStatus};
use tools::search::{SearchResult, WebSearcher};
use voice::VoiceClient;

#[derive(Debug)]
pub enum AppEvent {
    Tick,
    OllamaReplied {
        full_text: String,
        actions: Vec<ActionTag>,
        is_synthesis: bool,
    },
    OllamaError(String),
    WebSearchFinished {
        query: String,
        results: Result<Vec<SearchResult>, String>,
    },
    CommandFinished {
        command: String,
        output: Result<CommandOutput, String>,
    },
    VoiceRecordedAndTranscribed(Result<String, String>),
    TtsFinished,
    StatsUpdated(DeviceStats),
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load configuration
    let config = Config::load_or_default("config.toml");
    let mut app = App::new(config.clone());

    // 2. Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Set up event channel and services
    let (tx, mut rx) = mpsc::channel::<AppEvent>(100);
    let ollama_client = Arc::new(OllamaClient::new(config.ollama.clone()));
    let voice_client = Arc::new(VoiceClient::new(config.voice.clone()));
    let web_searcher = Arc::new(WebSearcher::new());
    let command_executor = Arc::new(CommandExecutor::new(config.tools.clone()));

    // Background tick generator
    let tx_tick = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(200));
        loop {
            interval.tick().await;
            if tx_tick.send(AppEvent::Tick).await.is_err() {
                break;
            }
        }
    });

    // Background health & device placement monitor (checks CPU/GPU allocation)
    let tx_health = tx.clone();
    let o_cli = ollama_client.clone();
    let v_cli = voice_client.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let ollama_ok = o_cli.check_connection().await;
            let voice_health = v_cli.check_health().await.ok();

            let stats = DeviceStats {
                ollama_connected: ollama_ok,
                voice_connected: voice_health.is_some(),
                whisper_loaded: voice_health.as_ref().map_or(false, |h| h.whisper_loaded),
                xtts_loaded: voice_health.as_ref().map_or(false, |h| h.xtts_loaded),
                vram_used_mb: voice_health.as_ref().map_or(0.0, |h| h.vram_allocated_mb),
                is_mock_voice: voice_health.as_ref().map_or(false, |h| h.is_mock),
            };

            if tx_health.send(AppEvent::StatsUpdated(stats)).await.is_err() {
                break;
            }
        }
    });

    // Initial check on launch
    {
        let tx_init = tx.clone();
        let o_cli = ollama_client.clone();
        let v_cli = voice_client.clone();
        tokio::spawn(async move {
            let ollama_ok = o_cli.check_connection().await;
            let voice_health = v_cli.check_health().await.ok();
            let stats = DeviceStats {
                ollama_connected: ollama_ok,
                voice_connected: voice_health.is_some(),
                whisper_loaded: voice_health.as_ref().map_or(false, |h| h.whisper_loaded),
                xtts_loaded: voice_health.as_ref().map_or(false, |h| h.xtts_loaded),
                vram_used_mb: voice_health.as_ref().map_or(0.0, |h| h.vram_allocated_mb),
                is_mock_voice: voice_health.as_ref().map_or(false, |h| h.is_mock),
            };
            let _ = tx_init.send(AppEvent::StatsUpdated(stats)).await;
        });
    }

    // 4. Main Event Loop
    while !app.should_quit {
        // Draw the UI
        terminal.draw(|f| ui::render(f, &app))?;

        // Handle terminal inputs (non-blocking with 30ms timeout)
        if event::poll(Duration::from_millis(30))? {
            let ev = event::read()?;
            if let Event::Mouse(mouse) = ev {
                match mouse.kind {
                    crossterm::event::MouseEventKind::ScrollUp => {
                        app.scroll_up(3);
                    }
                    crossterm::event::MouseEventKind::ScrollDown => {
                        app.scroll_down(3);
                    }
                    _ => {}
                }
            } else if let Event::Key(key) = ev {
                // If a confirmation modal is open:
                if let Some(pending) = app.pending_confirmation.clone() {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            // User approved once
                            app.pending_confirmation = None;
                            app.status = SystemStatus::Executing(pending.command.clone());
                            let tx_cmd = tx.clone();
                            let cmd_ex = command_executor.clone();
                            let cmd = pending.command.clone();
                            tokio::spawn(async move {
                                let res = cmd_ex.execute(&cmd).await.map_err(|e| e.to_string());
                                let _ = tx_cmd.send(AppEvent::CommandFinished {
                                    command: cmd,
                                    output: res,
                                }).await;
                            });
                        }
                        KeyCode::Char('a') | KeyCode::Char('A') => {
                            // User approved and added to session whitelist
                            app.config.tools.whitelist.push(pending.command.clone());
                            app.pending_confirmation = None;
                            app.status = SystemStatus::Executing(pending.command.clone());
                            let tx_cmd = tx.clone();
                            let cmd_ex = command_executor.clone();
                            let cmd = pending.command.clone();
                            tokio::spawn(async move {
                                let res = cmd_ex.execute(&cmd).await.map_err(|e| e.to_string());
                                let _ = tx_cmd.send(AppEvent::CommandFinished {
                                    command: cmd,
                                    output: res,
                                }).await;
                            });
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            // User denied command
                            app.pending_confirmation = None;
                            app.log_command(
                                pending.command.clone(),
                                Some(1),
                                "Execution denied by Boss".to_string(),
                                0,
                                false,
                            );
                            app.status = SystemStatus::Thinking;

                            // Inform Ollama about rejection
                            let notice = format!(
                                "[SECURITY_NOTICE: The command '{}' was rejected by Sir] Inform Sir with dry wit. Do NOT output [EXECUTE: ...] or [SEARCH: ...] tags.",
                                pending.command
                            );
                            app.chat_history.push(ChatMessage {
                                role: "user".to_string(),
                                content: notice,
                                tool_calls: None,
                            });

                            let tx_o = tx.clone();
                            let o_cli = ollama_client.clone();
                            let history = app.chat_history.clone();
                            tokio::spawn(async move {
                                match o_cli.chat(history).await {
                                    Ok(resp) => {
                                        let actions = OllamaClient::extract_actions(&resp.message.content);
                                        let _ = tx_o.send(AppEvent::OllamaReplied {
                                            full_text: resp.message.content,
                                            actions,
                                            is_synthesis: true,
                                        }).await;
                                    }
                                    Err(e) => {
                                        let _ = tx_o.send(AppEvent::OllamaError(e.to_string())).await;
                                    }
                                }
                            });
                        }
                        _ => {}
                    }
                } else {
                    // Regular interactive mode
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            app.should_quit = true;
                        }
                        (KeyCode::Tab, _) | (KeyCode::F(2), _) => {
                            app.toggle_input_mode();
                        }
                        (KeyCode::Esc, _) => {
                            app.input_text.clear();
                            app.cursor_position = 0;
                            app.is_recording = false;
                        }
                        // Global Scrolling Keybindings
                        (KeyCode::PageUp, _) => {
                            app.scroll_up(10);
                        }
                        (KeyCode::PageDown, _) => {
                            app.scroll_down(10);
                        }
                        (KeyCode::Home, _) => {
                            app.scroll_to_top();
                        }
                        (KeyCode::End, _) => {
                            app.scroll_to_bottom();
                        }
                        (KeyCode::Up, KeyModifiers::ALT) | (KeyCode::Up, KeyModifiers::CONTROL) => {
                            app.scroll_up(3);
                        }
                        (KeyCode::Down, KeyModifiers::ALT) | (KeyCode::Down, KeyModifiers::CONTROL) => {
                            app.scroll_down(3);
                        }
                        _ => match app.input_mode {
                            InputMode::Text => match key.code {
                                KeyCode::Up => {
                                    app.scroll_up(3);
                                }
                                KeyCode::Down => {
                                    app.scroll_down(3);
                                }
                                KeyCode::Enter => {
                                    let prompt = app.input_text.trim().to_string();
                                    if !prompt.is_empty() {
                                        app.add_user_message(prompt.clone(), false);
                                        app.input_text.clear();
                                        app.cursor_position = 0;
                                        app.status = SystemStatus::Thinking;

                                        let tx_o = tx.clone();
                                        let o_cli = ollama_client.clone();
                                        let history = app.chat_history.clone();

                                        tokio::spawn(async move {
                                            match o_cli.chat(history).await {
                                                Ok(resp) => {
                                                    let actions = OllamaClient::extract_actions(
                                                        &resp.message.content,
                                                    );
                                                    let _ = tx_o.send(AppEvent::OllamaReplied {
                                                        full_text: resp.message.content,
                                                        actions,
                                                        is_synthesis: false,
                                                    }).await;
                                                }
                                                Err(e) => {
                                                    let _ = tx_o
                                                        .send(AppEvent::OllamaError(e.to_string()))
                                                        .await;
                                                }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Char(c) => {
                                    app.input_text.insert(app.cursor_position, c);
                                    app.cursor_position += 1;
                                }
                                KeyCode::Backspace => {
                                    if app.cursor_position > 0 {
                                        app.cursor_position -= 1;
                                        app.input_text.remove(app.cursor_position);
                                    }
                                }
                                KeyCode::Left => {
                                    if app.cursor_position > 0 {
                                        app.cursor_position -= 1;
                                    }
                                }
                                KeyCode::Right => {
                                    if app.cursor_position < app.input_text.len() {
                                        app.cursor_position += 1;
                                    }
                                }
                                _ => {}
                            },
                            InputMode::Voice => match key.code {
                                KeyCode::Up => {
                                    app.scroll_up(3);
                                }
                                KeyCode::Down => {
                                    app.scroll_down(3);
                                }
                                KeyCode::Enter | KeyCode::Char(' ') => {
                                    if !app.is_recording {
                                        // Start microphone recording and transcription
                                        app.is_recording = true;
                                        app.status = SystemStatus::Listening;

                                        let tx_v = tx.clone();
                                        let v_cli = voice_client.clone();
                                        let duration_sec = app.config.voice.record_duration_sec;

                                        tokio::spawn(async move {
                                            let res = v_cli
                                                .listen_and_transcribe(duration_sec)
                                                .await
                                                .map_err(|e| e.to_string());
                                            let _ = tx_v
                                                .send(AppEvent::VoiceRecordedAndTranscribed(res))
                                                .await;
                                        });
                                    } else {
                                        // User pressed again to stop early
                                        app.is_recording = false;
                                        app.status = SystemStatus::Transcribing;
                                    }
                                }
                                _ => {}
                            },
                        },
                    }
                }
            }
        }

        // Process asynchronous events from background workers
        while let Ok(event) = rx.try_recv() {
            match event {
                AppEvent::Tick => {
                    app.on_tick();
                }
                AppEvent::StatsUpdated(stats) => {
                    app.device_stats = stats;
                }
                AppEvent::VoiceRecordedAndTranscribed(res) => {
                    app.is_recording = false;
                    match res {
                        Ok(transcription) => {
                            let text = transcription.trim().to_string();
                            if !text.is_empty() {
                                app.add_user_message(text.clone(), true);
                                app.status = SystemStatus::Thinking;

                                let tx_o = tx.clone();
                                let o_cli = ollama_client.clone();
                                let history = app.chat_history.clone();

                                tokio::spawn(async move {
                                    match o_cli.chat(history).await {
                                        Ok(resp) => {
                                            let actions = OllamaClient::extract_actions(
                                                &resp.message.content,
                                            );
                                            let _ = tx_o.send(AppEvent::OllamaReplied {
                                                full_text: resp.message.content,
                                                actions,
                                                is_synthesis: false,
                                            }).await;
                                        }
                                        Err(e) => {
                                            let _ = tx_o
                                                .send(AppEvent::OllamaError(e.to_string()))
                                                .await;
                                        }
                                    }
                                });
                            } else {
                                app.status = SystemStatus::Idle;
                                app.add_system_message("No speech detected. Please speak clearly into the microphone.".into());
                            }
                        }
                        Err(err) => {
                            app.status = SystemStatus::Error(err.clone());
                            app.add_system_message(format!("Whisper STT failure: {}. Check that the voice service is running.", err));
                        }
                    }
                }
                AppEvent::OllamaReplied {
                    full_text,
                    actions,
                    is_synthesis,
                } => {
                    if actions.is_empty() || is_synthesis {
                        // Direct dialogue response or post-tool synthesis:
                        // Display final refined butler response and play TTS
                        let visible_text = OllamaClient::strip_thinking(&full_text);
                        let clean_speech = OllamaClient::clean_text_for_speech(&full_text);

                        // Also strip action tags from the visible text for display
                        let re_exec = regex::Regex::new(r"\[EXECUTE:\s*[^\]]+\]").unwrap();
                        let d = re_exec.replace_all(&visible_text, "");
                        let re_search = regex::Regex::new(r"\[SEARCH:\s*[^\]]+\]").unwrap();
                        let display_text = re_search.replace_all(&d, "").trim().to_string();

                        if !display_text.is_empty() {
                            app.add_assistant_message(display_text, app.input_mode == InputMode::Voice);
                        }

                        // Trigger XTTS speech on GPU
                        if (app.input_mode == InputMode::Voice || app.config.voice.auto_play_tts)
                            && !clean_speech.is_empty()
                        {
                            app.status = SystemStatus::Speaking;
                            let tx_tts = tx.clone();
                            let v_cli = voice_client.clone();
                            tokio::spawn(async move {
                                let _ = v_cli.speak(&clean_speech).await;
                                let _ = tx_tts.send(AppEvent::TtsFinished).await;
                            });
                        } else {
                            app.status = SystemStatus::Idle;
                        }
                    } else {
                        // Action tag detected from primary prompt!
                        // 1. Record the assistant's action into chat_history so dialogue turn integrity is preserved
                        app.chat_history.push(ChatMessage {
                            role: "assistant".into(),
                            content: full_text.clone(),
                            tool_calls: None,
                        });

                        // 2. Display polite acknowledgment if text accompanies the tag
                        let ack = OllamaClient::clean_text_for_speech(&full_text);
                        if !ack.is_empty() {
                            app.messages.push(DisplayMessage {
                                role: MessageRole::Alfred,
                                content: ack,
                                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                                is_voice: false,
                            });
                        }

                        // 3. Dispatch the appropriate tool
                        match &actions[0] {
                            ActionTag::Search(query) => {
                                app.status = SystemStatus::Searching(query.clone());
                                let tx_s = tx.clone();
                                let ws = web_searcher.clone();
                                let q = query.clone();
                                tokio::spawn(async move {
                                    let res = ws.search(&q).await.map_err(|e| e.to_string());
                                    let _ = tx_s.send(AppEvent::WebSearchFinished {
                                        query: q,
                                        results: res,
                                    }).await;
                                });
                            }
                            ActionTag::Execute(cmd) => {
                                match command_executor.evaluate_guardrails(cmd) {
                                    GuardrailStatus::AllowedAuto => {
                                        app.status = SystemStatus::Executing(cmd.clone());
                                        let tx_cmd = tx.clone();
                                        let cmd_ex = command_executor.clone();
                                        let c = cmd.clone();
                                        tokio::spawn(async move {
                                            let res = cmd_ex.execute(&c).await.map_err(|e| e.to_string());
                                            let _ = tx_cmd.send(AppEvent::CommandFinished {
                                                command: c,
                                                output: res,
                                            }).await;
                                        });
                                    }
                                    GuardrailStatus::RequiresConfirmation => {
                                        app.status = SystemStatus::AwaitingConfirmation;
                                        app.pending_confirmation = Some(PendingConfirmation {
                                            command: cmd.clone(),
                                            reason: "Command is not in auto-approved whitelist.".into(),
                                        });
                                    }
                                    GuardrailStatus::Blocked(reason) => {
                                        app.log_command(
                                            cmd.clone(),
                                            Some(1),
                                            format!("Guardrail blocked: {}", reason),
                                            0,
                                            false,
                                        );
                                        app.status = SystemStatus::Thinking;

                                        let notice = format!(
                                            "[GUARDRAIL_BLOCKED: Command '{}' violates safety: {}] Inform Sir with dry wit. Do NOT output [EXECUTE: ...] or [SEARCH: ...] tags.",
                                            cmd, reason
                                        );
                                        app.chat_history.push(ChatMessage {
                                            role: "user".to_string(),
                                            content: notice,
                                            tool_calls: None,
                                        });

                                        let tx_o = tx.clone();
                                        let o_cli = ollama_client.clone();
                                        let history = app.chat_history.clone();
                                        tokio::spawn(async move {
                                            match o_cli.chat(history).await {
                                                Ok(resp) => {
                                                    let actions = OllamaClient::extract_actions(
                                                        &resp.message.content,
                                                    );
                                                    let _ = tx_o.send(AppEvent::OllamaReplied {
                                                        full_text: resp.message.content,
                                                        actions,
                                                        is_synthesis: true,
                                                    }).await;
                                                }
                                                Err(e) => {
                                                    let _ = tx_o
                                                        .send(AppEvent::OllamaError(e.to_string()))
                                                        .await;
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                AppEvent::WebSearchFinished { query, results } => {
                    let tool_content = match &results {
                        Ok(list) => {
                            let count = list.len();
                            let summary = if let Some(first) = list.first() {
                                first.snippet.clone()
                            } else {
                                "No snippets found".to_string()
                            };

                            app.log_search(query.clone(), count, summary);
                            app.add_tool_message(format!(
                                "Web search retrieved {} result(s) for '{}'",
                                count, query
                            ));

                            let mut formatted_results = String::new();
                            for item in list.iter().take(3) {
                                formatted_results.push_str(&format!(
                                    "Title: {}\nSnippet: {}\nURL: {}\n\n",
                                    item.title, item.snippet, item.link
                                ));
                            }

                            format!(
                                "[SEARCH_RESULTS for '{}':\n{}] Please provide your refined butler answer to Sir based on this. Do NOT output [SEARCH: ...] or [EXECUTE: ...] again.",
                                query, formatted_results
                            )
                        }
                        Err(e) => {
                            app.log_search(query.clone(), 0, format!("Search failed: {}", e));
                            app.add_tool_message(format!(
                                "Web search failed for '{}': {}",
                                query, e
                            ));
                            format!(
                                "[SEARCH_ERROR: Failed to retrieve web results for '{}': {}] Inform Sir with dry wit. Do NOT output [SEARCH: ...] or [EXECUTE: ...] again.",
                                query, e
                            )
                        }
                    };

                    app.chat_history.push(ChatMessage {
                        role: "user".to_string(),
                        content: tool_content,
                        tool_calls: None,
                    });

                    app.status = SystemStatus::Thinking;
                    let tx_o = tx.clone();
                    let o_cli = ollama_client.clone();
                    let history = app.chat_history.clone();
                    tokio::spawn(async move {
                        match o_cli.chat(history).await {
                            Ok(resp) => {
                                let actions = OllamaClient::extract_actions(&resp.message.content);
                                let _ = tx_o.send(AppEvent::OllamaReplied {
                                    full_text: resp.message.content,
                                    actions,
                                    is_synthesis: true,
                                }).await;
                            }
                            Err(e) => {
                                let _ = tx_o.send(AppEvent::OllamaError(e.to_string())).await;
                            }
                        }
                    });
                }
                AppEvent::CommandFinished { command, output } => {
                    let tool_content = match &output {
                        Ok(cmd_out) => {
                            app.log_command(
                                command.clone(),
                                cmd_out.exit_code,
                                cmd_out.stdout.clone(),
                                cmd_out.duration_ms,
                                true,
                            );
                            app.add_tool_message(format!(
                                "Executed '{}' (exit: {:?}, {}ms)",
                                command, cmd_out.exit_code, cmd_out.duration_ms
                            ));

                            format!(
                                "[COMMAND_OUTPUT for '{}' (exit: {:?}):\n{}] Acknowledge this to Sir in your formal butler manner. Do NOT output [EXECUTE: ...] or [SEARCH: ...] again.",
                                command, cmd_out.exit_code, cmd_out.stdout
                            )
                        }
                        Err(e) => {
                            app.log_command(command.clone(), Some(1), e.clone(), 0, false);
                            app.add_tool_message(format!(
                                "Execution failed for '{}': {}",
                                command, e
                            ));
                            format!(
                                "[COMMAND_ERROR for '{}': {}] Inform Sir with dry wit. Do NOT output [EXECUTE: ...] or [SEARCH: ...] again.",
                                command, e
                            )
                        }
                    };

                    app.chat_history.push(ChatMessage {
                        role: "user".to_string(),
                        content: tool_content,
                        tool_calls: None,
                    });

                    app.status = SystemStatus::Thinking;
                    let tx_o = tx.clone();
                    let o_cli = ollama_client.clone();
                    let history = app.chat_history.clone();
                    tokio::spawn(async move {
                        match o_cli.chat(history).await {
                            Ok(resp) => {
                                let actions = OllamaClient::extract_actions(&resp.message.content);
                                let _ = tx_o.send(AppEvent::OllamaReplied {
                                    full_text: resp.message.content,
                                    actions,
                                    is_synthesis: true,
                                }).await;
                            }
                            Err(e) => {
                                let _ = tx_o.send(AppEvent::OllamaError(e.to_string())).await;
                            }
                        }
                    });
                }
                AppEvent::OllamaError(err) => {
                    app.status = SystemStatus::Error(err.clone());
                    app.add_system_message(format!(
                        "Ollama communication failure: {}. Verify that 'ollama serve' is running on CPU.",
                        err
                    ));
                }
                AppEvent::TtsFinished => {
                    app.status = SystemStatus::Idle;
                }
            }
        }
    }

    // 5. Restore terminal on quit
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    println!("Alfred has retired to the pantry. Have a pleasant evening, Sir.");
    Ok(())
}
