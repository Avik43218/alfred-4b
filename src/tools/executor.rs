use anyhow::{bail, Result};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

use crate::config::ToolsConfig;

#[derive(Debug, Clone)]
pub enum GuardrailStatus {
    AllowedAuto,
    RequiresConfirmation,
    Blocked(String),
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u128,
}

pub struct CommandExecutor {
    config: ToolsConfig,
}

impl CommandExecutor {
    pub fn new(config: ToolsConfig) -> Self {
        Self { config }
    }

    /// Checks command against whitelist and blacklist guardrails
    pub fn evaluate_guardrails(&self, cmd: &str) -> GuardrailStatus {
        let trimmed = cmd.trim();

        // 1. Check blacklist first (hard block)
        for blocked in &self.config.blacklist {
            if trimmed.contains(blocked) {
                return GuardrailStatus::Blocked(format!(
                    "Command contains dangerous pattern '{}'",
                    blocked
                ));
            }
        }

        // 2. Check whitelist (safe commands run without user prompt)
        for allowed in &self.config.whitelist {
            if trimmed == allowed || trimmed.starts_with(&format!("{} ", allowed)) {
                return GuardrailStatus::AllowedAuto;
            }
        }

        // 3. If confirmation is enabled and command is not in whitelist, require confirmation
        if self.config.require_confirmation {
            GuardrailStatus::RequiresConfirmation
        } else {
            GuardrailStatus::AllowedAuto
        }
    }

    /// Executes command with timeout and buffer truncation
    pub async fn execute(&self, cmd: &str) -> Result<CommandOutput> {
        let start = std::time::Instant::now();
        let trimmed = cmd.trim();

        // Easter egg from Modelfile: tea brewing
        if trimmed == "brew-tea" {
            return Ok(CommandOutput {
                command: cmd.to_string(),
                exit_code: Some(0),
                stdout: "Earl Grey, hot. Steeping at 95°C for 4 minutes. A proper cup, Sir."
                    .to_string(),
                stderr: String::new(),
                duration_ms: start.elapsed().as_millis(),
            });
        }

        let timeout_dur = Duration::from_secs(self.config.command_timeout_sec);

        let child = Command::new("bash")
            .arg("-c")
            .arg(trimmed)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn command '{}': {}", trimmed, e))?;

        let output_res = timeout(timeout_dur, child.wait_with_output()).await;

        match output_res {
            Ok(Ok(output)) => {
                let duration_ms = start.elapsed().as_millis();
                let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let mut stderr = String::from_utf8_lossy(&output.stderr).to_string();

                // Cap output buffer at 4KB to avoid context explosion
                const MAX_CHARS: usize = 4096;
                if stdout.len() > MAX_CHARS {
                    stdout.truncate(MAX_CHARS);
                    stdout.push_str("\n... [Output truncated to 4KB]");
                }
                if stderr.len() > MAX_CHARS {
                    stderr.truncate(MAX_CHARS);
                    stderr.push_str("\n... [Stderr truncated to 4KB]");
                }

                Ok(CommandOutput {
                    command: cmd.to_string(),
                    exit_code: output.status.code(),
                    stdout: stdout.trim().to_string(),
                    stderr: stderr.trim().to_string(),
                    duration_ms,
                })
            }
            Ok(Err(e)) => bail!("Error executing command: {}", e),
            Err(_) => bail!(
                "Command execution timed out after {} seconds",
                self.config.command_timeout_sec
            ),
        }
    }
}
