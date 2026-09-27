use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceHealthResponse {
    pub status: String,
    pub whisper_loaded: bool,
    pub whisper_device: String,
    pub xtts_loaded: bool,
    pub xtts_device: String,
    pub vram_allocated_mb: f32,
    pub vram_reserved_mb: f32,
    #[serde(default)]
    pub is_mock: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResponse {
    pub text: String,
    pub duration_sec: Option<f32>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsSpeakRequest {
    pub text: String,
    pub play_audio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsSpeakResponse {
    pub status: String,
    pub duration_sec: Option<f32>,
    pub played: bool,
}
