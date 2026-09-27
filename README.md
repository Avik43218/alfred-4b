# 🎩 A.L.F.R.E.D. (Automated Local Faithful Reasoning & Execution Device)

**A.L.F.R.E.D.** is a high-performance, terminal-based local AI digital butler written in **Rust** (Ratatui TUI) and **Python**, designed with strict memory budgets for laptops with **4GB of VRAM**.

Alfred combines local speech-to-text (**Faster-Whisper on GPU**), text-to-speech (**Coqui XTTS on GPU**), local reasoning (**Ollama LLM on CPU/System RAM**), live web search, and guarded shell command execution.

---

## 🏛️ System Architecture

```text
               +-------------------------------------------------------+
               |                A.L.F.R.E.D. Rust TUI                  |
               |             (Ratatui + Crossterm + Tokio)             |
               +---------------------------+---------------------------+
                                           |
                    +----------------------+----------------------+
                    | [Tab] Input Mode                            |
           +--------v--------+                           +--------v--------+
           |   Text Mode     |                           |   Voice Mode    |
           | (Keyboard input)|                           |  (Microphone)   |
           +--------+--------+                           +--------+--------+
                    |                                             |
                    |                                             v [HTTP: 8765]
                    |                                    +-----------------+
                    |                                    | Faster-Whisper  |
                    |                                    |  (CUDA:0 - GPU) |
                    |                                    +--------+--------+
                    |                                             | Transcribed
                    +---------------------> <---------------------+ Text
                                           |
                                           v [HTTP: 11434, num_gpu: 0]
                         +-----------------------------------+
                         |            Ollama LLM             |
                         |      (CPU / System RAM Only)      |
                         |  Model: alfred / qwen2.5 / llama  |
                         +-----------------+-----------------+
                                           |
                       +-------------------+-------------------+
                       |                                       |
                       v [Action: SEARCH]                      v [Action: EXECUTE]
             +--------------------+                 +--------------------+
             |   Web Search Tool  |                 |  Command Executor  |
             |  (DuckDuckGo Live) |                 | (Safety Guardrails)|
             +---------+----------+                 +---------+----------+
                       |                                       |
                       +-------------------+-------------------+
                                           | Results fed back to Ollama
                                           v
                         +-----------------------------------+
                         |      Final Butler Response        |
                         +-----------------+-----------------+
                                           |
                    +----------------------+----------------------+
                    |                                             |
                    v                                             v [HTTP: 8765]
          +-------------------+                         +-------------------+
          |  TUI Dialogue Log |                         |  Coqui XTTS v2    |
          |  (Visual Output)  |                         |  (CUDA:0 - GPU)   |
          +-------------------+                         +-------------------+
                                                                  |
                                                                  v
                                                        System Audio Playback
```

---

## ⚡ 4GB VRAM Budget & Hardware Allocation (Critical)

On systems with 4GB VRAM (such as an NVIDIA RTX 3050 Laptop GPU) and 16GB system RAM, loading an LLM onto VRAM alongside speech models will cause out-of-memory (OOM) crashes.

A.L.F.R.E.D. implements a strict **hardware partition**:

| Component | Target Device | Framework | VRAM Footprint | RAM Footprint |
| :--- | :--- | :--- | :--- | :--- |
| **STT (Speech-to-Text)** | **GPU (CUDA:0)** | Faster-Whisper (`base.en` / `small.en`) | **~300 MB** | ~100 MB |
| **TTS (Text-to-Speech)** | **GPU (CUDA:0)** | Coqui XTTS v2 (`float16`) | **~2.8 GB** | ~500 MB |
| **LLM (Reasoning)** | **CPU (System RAM)** | Ollama (`qwen2.5:3b` / `alfred`) | **0 MB (0 Layers)** | **~2.5 GB** |
| **Total Allocation** | — | — | **~3.1 GB / 4.0 GB** | **~3.1 GB / 16 GB** |

> [!IMPORTANT]
> **VRAM Protection Guarantees:**
> 1. When launching Ollama, the environment variable `OLLAMA_NUM_GPU=0` is set.
> 2. Every HTTP request sent from the Rust application to `/api/chat` passes `"options": { "num_gpu": 0 }`. Even if Ollama was started normally, this API-level override ensures zero transformer layers are offloaded to VRAM.
> 3. Coqui XTTS and Faster-Whisper run with FP16/int8 quantization on CUDA:0, leaving ~900MB of VRAM headroom.

---

## 🛡️ Tools & Safety Guardrails

### 1. Live Web Search Tool
- Queries DuckDuckGo Instant Answers and HTML search endpoints without requiring paid API keys.
- Extracts article titles, snippets, and source URLs.
- Automatically feeds live context back into the conversation for current events, documentation, or weather.

### 2. Command Execution Guardrails
- **Safe Whitelist (Auto-run):** Safe inspection commands (`ls`, `dir`, `pwd`, `date`, `uptime`, `whoami`, `cat`, `head`, `tail`, `grep`, `find`, `uname`, `free`, `df`, `ps`, `git status`, `echo`, `which`, `brew-tea`).
- **Blacklist (Blocked):** Destructive commands (`rm -rf /`, `rm -rf ~`, `mkfs`, `dd`, `shutdown`, `reboot`, `sudo`, fork bombs).
- **Interactive TUI Confirmation Modal:** Any command not on the whitelist triggers a safety dialog popup. You can approve once (`[Y]`), approve and add to session whitelist (`[A]`), or deny (`[N]`).
- **Buffer & Timeout Limits:** Commands are capped at 10 seconds execution time and 4KB buffer truncation to avoid hanging or context explosion.

---

## 🚀 Quick Start Guide

### Step 1: Install System Prerequisites
On Fedora/RHEL:
```bash
sudo dnf install alsa-utils pulseaudio-utils ffmpeg
```
On Ubuntu/Debian:
```bash
sudo apt update && sudo apt install alsa-utils pulseaudio-utils ffmpeg
```

### Step 2: Configure Ollama (CPU Mode)
Build the custom A.L.F.R.E.D. butler model from `Modelfile`:
```bash
bash scripts/setup_ollama.sh
```

### Step 3: Setup Python Voice Environment (GPU Mode)
Create the virtual environment and install PyTorch CUDA, Faster-Whisper, and Coqui XTTS:
```bash
bash scripts/setup_env.sh
```

*(Note: To test without downloading full model weights, launch with `--mock` flag).*

### Step 4: Run A.L.F.R.E.D.
Launch all background services (Ollama on CPU, Voice on GPU) and start the TUI:
```bash
./start_alfred.sh
```
*(Or test in simulated mode without downloading heavy weights: `./start_alfred.sh --mock`)*

Or run the voice service and TUI independently:
```bash
# Terminal 1: Voice Service (GPU)
bash scripts/run_voice_service.sh

# Terminal 2: Rust TUI
cargo run --release
```

---

## 🎮 Keybindings & Controls

| Key | Action |
| :--- | :--- |
| `[Tab]` / `[F2]` | Toggle between **[TEXT]** and **[VOICE]** input modes |
| `[Enter]` | In Text mode: Submit prompt to Alfred |
| `[Space]` / `[Enter]` | In Voice mode: Start / stop microphone recording for Whisper STT |
| `[Esc]` | Clear input buffer / cancel recording / dismiss modal |
| `[Y]` | Approve command execution (in confirmation modal) |
| `[A]` | Approve and add command to session whitelist |
| `[N]` | Deny command execution |
| `[Ctrl + C]` | Gracefully quit application |

---

## ⚙️ Configuration (`config.toml`)

All system behaviors, hardware allocations, and tool guardrails are configurable in `config.toml`:

```toml
[hardware]
llm_device = "cpu"
stt_device = "cuda"
tts_device = "cuda"
vram_limit_mb = 4096
enforce_cpu_llm = true

[ollama]
host = "http://127.0.0.1:11434"
model = "alfred"
fallback_model = "qwen2.5:3b"
num_gpu = 0
num_threads = 8

[voice]
service_url = "http://127.0.0.1:8765"
whisper_model = "base.en"
whisper_compute_type = "float16"
xtts_model = "tts_models/multilingual/multi-dataset/xtts_v2"
auto_play_tts = true
record_duration_sec = 6

[tools]
web_search_enabled = true
command_exec_enabled = true
require_confirmation = true
whitelist = ["ls", "pwd", "date", "uptime", "free", "df", "ps", "brew-tea"]
```
