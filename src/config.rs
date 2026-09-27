use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub system: SystemConfig,
    pub hardware: HardwareConfig,
    pub ollama: OllamaConfig,
    pub voice: VoiceConfig,
    pub tools: ToolsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub name: String,
    pub version: String,
    pub boss_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    pub llm_device: String,
    pub stt_device: String,
    pub tts_device: String,
    pub vram_limit_mb: u32,
    pub enforce_cpu_llm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub host: String,
    pub model: String,
    pub fallback_model: String,
    pub num_gpu: i32,
    pub num_threads: u32,
    pub temperature: f32,
    pub num_ctx: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub service_url: String,
    pub whisper_model: String,
    pub whisper_compute_type: String,
    pub xtts_model: String,
    pub language: String,
    pub auto_play_tts: bool,
    pub record_duration_sec: u64,
    #[serde(default)]
    pub speaker_wav: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    pub web_search_enabled: bool,
    pub command_exec_enabled: bool,
    pub require_confirmation: bool,
    pub command_timeout_sec: u64,
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            system: SystemConfig {
                name: "A.L.F.R.E.D.".to_string(),
                version: "1.0.0".to_string(),
                boss_title: "Sir".to_string(),
            },
            hardware: HardwareConfig {
                llm_device: "cpu".to_string(),
                stt_device: "cuda".to_string(),
                tts_device: "cuda".to_string(),
                vram_limit_mb: 4096,
                enforce_cpu_llm: true,
            },
            ollama: OllamaConfig {
                host: "http://127.0.0.1:11434".to_string(),
                model: "alfred:latest".to_string(),
                fallback_model: "qwen3:4b-instruct".to_string(),
                num_gpu: 0,
                num_threads: 8,
                temperature: 0.7,
                num_ctx: 4096,
            },
            voice: VoiceConfig {
                service_url: "http://127.0.0.1:8765".to_string(),
                whisper_model: "base.en".to_string(),
                whisper_compute_type: "float16".to_string(),
                xtts_model: "tts_models/multilingual/multi-dataset/xtts_v2".to_string(),
                language: "en".to_string(),
                auto_play_tts: true,
                record_duration_sec: 6,
                speaker_wav: Some("assets/voices/alfred.wav".to_string()),
            },
            tools: ToolsConfig {
                web_search_enabled: true,
                command_exec_enabled: true,
                require_confirmation: true,
                command_timeout_sec: 10,
                whitelist: vec![
                    "ls".into(), "dir".into(), "pwd".into(), "date".into(),
                    "uptime".into(), "whoami".into(), "cat".into(), "head".into(),
                    "tail".into(), "grep".into(), "find".into(), "uname".into(),
                    "free".into(), "df".into(), "ps".into(), "git status".into(),
                    "echo".into(), "which".into(), "brew-tea".into(),
                ],
                blacklist: vec![
                    "rm -rf /".into(), "rm -rf ~".into(), "mkfs".into(),
                    "dd".into(), "shutdown".into(), "reboot".into(),
                    "sudo".into(), ":(){ :|:& };:".into(),
                ],
            },
        }
    }
}

impl Config {
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(config) = toml::from_str(&content) {
                return config;
            }
        }
        Config::default()
    }
}
