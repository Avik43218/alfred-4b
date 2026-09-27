use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

use super::types::{TtsSpeakRequest, TtsSpeakResponse, TranscriptionResponse, VoiceHealthResponse};
use crate::config::VoiceConfig;

#[derive(Clone)]
pub struct VoiceClient {
    client: Client,
    config: VoiceConfig,
}

impl VoiceClient {
    pub fn new(config: VoiceConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();
        Self { client, config }
    }

    pub async fn check_health(&self) -> Result<VoiceHealthResponse> {
        let url = format!("{}/health", self.config.service_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to Voice Service")?;

        let health: VoiceHealthResponse = resp
            .json()
            .await
            .context("Failed to parse Voice health response")?;
        Ok(health)
    }

    /// Triggers the GPU-accelerated Whisper STT to record and transcribe speech
    pub async fn listen_and_transcribe(&self, duration_sec: u64) -> Result<String> {
        let url = format!("{}/stt/listen", self.config.service_url);
        let payload = serde_json::json!({
            "duration_sec": duration_sec,
            "language": self.config.language,
        });

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to request transcription from Voice Service")?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("Voice service STT error: {}", err);
        }

        let result: TranscriptionResponse = resp
            .json()
            .await
            .context("Failed to parse transcription response")?;

        Ok(result.text.trim().to_string())
    }

    /// Triggers GPU-accelerated Coqui XTTS to synthesize and play spoken response
    pub async fn speak(&self, text: &str) -> Result<()> {
        let url = format!("{}/tts/speak", self.config.service_url);
        let payload = TtsSpeakRequest {
            text: text.to_string(),
            play_audio: self.config.auto_play_tts,
        };

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send speech request to XTTS service")?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("Voice service TTS error: {}", err);
        }

        let _result: TtsSpeakResponse = resp
            .json()
            .await
            .context("Failed to parse TTS response")?;

        Ok(())
    }
}
