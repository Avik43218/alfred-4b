use anyhow::{Context, Result};
use regex::Regex;
use reqwest::Client;
use std::time::Duration;

use super::types::{ChatMessage, ChatRequest, ChatResponse, ModelListResponse, OllamaOptions};
use crate::config::OllamaConfig;

#[derive(Debug, Clone, PartialEq)]
pub enum ActionTag {
    Execute(String),
    Search(String),
}

#[derive(Clone)]
pub struct OllamaClient {
    client: Client,
    config: OllamaConfig,
}

impl OllamaClient {
    pub fn new(config: OllamaConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(90))
            .build()
            .unwrap_or_default();
        Self { client, config }
    }

    pub async fn check_connection(&self) -> bool {
        let url = format!("{}/api/tags", self.config.host);
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.config.host);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        if !resp.status().is_success() {
            return Ok(vec![]);
        }

        let body: ModelListResponse = resp.json().await.context("Failed to parse Ollama model list")?;
        Ok(body.models.into_iter().map(|m| m.name).collect())
    }

    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatResponse> {
        let url = format!("{}/api/chat", self.config.host);

        // First check available models to select preferred or fallback
        let target_model = match self.list_models().await {
            Ok(models) => {
                if models.iter().any(|m| m.starts_with(&self.config.model)) {
                    self.config.model.clone()
                } else if models.iter().any(|m| m.starts_with(&self.config.fallback_model)) {
                    self.config.fallback_model.clone()
                } else if let Some(first) = models.first() {
                    first.clone()
                } else {
                    self.config.model.clone()
                }
            }
            Err(_) => self.config.model.clone(),
        };

        // Note: num_gpu: 0 explicitly guarantees the Ollama model runs strictly on
        // system RAM / CPU, reserving the entire 4GB VRAM for Whisper and XTTS.
        let request = ChatRequest {
            model: target_model,
            messages,
            stream: false,
            options: OllamaOptions {
                num_gpu: self.config.num_gpu,
                num_thread: self.config.num_threads,
                temperature: self.config.temperature,
                num_ctx: self.config.num_ctx,
            },
            tools: None,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send chat request to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama error (HTTP {}): {}", status, error_text);
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse Ollama chat response")?;

        Ok(chat_response)
    }

    /// Strips <think>...</think> reasoning blocks from Qwen3 / DeepSeek reasoning outputs
    pub fn strip_thinking(text: &str) -> String {
        let re_think = Regex::new(r"(?s)<think>.*?</think>").unwrap();
        let cleaned = re_think.replace_all(text, "");
        let re_unclosed = Regex::new(r"(?s)<think>.*").unwrap();
        let cleaned = re_unclosed.replace_all(&cleaned, "");
        cleaned.trim().to_string()
    }

    /// Parses command execution or web search action tags from model response.
    /// Supports tags specified in Modelfile:
    /// - [EXECUTE: <command>]
    /// - [SEARCH: <query>]
    pub fn extract_actions(text: &str) -> Vec<ActionTag> {
        let visible_text = Self::strip_thinking(text);
        let mut actions = Vec::new();

        let re_exec = Regex::new(r"\[EXECUTE:\s*([^\]]+)\]").unwrap();
        for cap in re_exec.captures_iter(&visible_text) {
            if let Some(cmd) = cap.get(1) {
                let cmd_str = cmd.as_str().trim().to_string();
                if !cmd_str.is_empty() {
                    actions.push(ActionTag::Execute(cmd_str));
                }
            }
        }

        let re_search = Regex::new(r"\[SEARCH:\s*([^\]]+)\]").unwrap();
        for cap in re_search.captures_iter(&visible_text) {
            if let Some(query) = cap.get(1) {
                let q_str = query.as_str().trim().to_string();
                if !q_str.is_empty() {
                    actions.push(ActionTag::Search(q_str));
                }
            }
        }

        actions
    }

    /// Strips thinking blocks and action tags from text to prepare speech-ready response for XTTS
    pub fn clean_text_for_speech(text: &str) -> String {
        let visible_text = Self::strip_thinking(text);
        let re_exec = Regex::new(r"\[EXECUTE:\s*[^\]]+\]").unwrap();
        let cleaned = re_exec.replace_all(&visible_text, "");
        let re_search = Regex::new(r"\[SEARCH:\s*[^\]]+\]").unwrap();
        let cleaned = re_search.replace_all(&cleaned, "");
        cleaned.trim().to_string()
    }
}
