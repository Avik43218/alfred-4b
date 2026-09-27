#!/usr/bin/env bash
# ==============================================================================
# A.L.F.R.E.D. Master Service Orchestrator & UI Launcher
# 
# 1. Launches Ollama in CPU mode (OLLAMA_NUM_GPU=0) in the background.
# 2. Launches the Voice Microservice (Whisper STT + XTTS on CUDA:0) in the background.
# 3. Verifies health of all background daemons.
# 4. Launches the Rust Ratatui TUI.
# 5. Automatically cleans up all background processes on exit.
# ==============================================================================
set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

LOG_DIR="$PROJECT_ROOT/logs"
mkdir -p "$LOG_DIR"

OLLAMA_LOG="$LOG_DIR/ollama.log"
VOICE_LOG="$LOG_DIR/voice_service.log"
VENV_DIR="$PROJECT_ROOT/.venv"

# Handle CLI options
if [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
    echo "Usage: ./start_alfred.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --mock      Run Voice service in simulated mock mode (no GPU model weights required)"
    echo "  --status    Check status of background services and exit"
    echo "  --stop      Stop all running A.L.F.R.E.D. background services"
    echo "  -h, --help  Show this help message"
    exit 0
fi

if [ "$1" == "--stop" ]; then
    echo "[*] Stopping any existing A.L.F.R.E.D. services..."
    pkill -f "voice_service/service.py" 2>/dev/null && echo "[+] Voice service stopped." || echo "[-] Voice service was not running."
    exit 0
fi

if [ "$1" == "--status" ]; then
    echo "=== A.L.F.R.E.D. Service Health Check ==="
    echo -n "1. Ollama (CPU): "
    if curl -s http://127.0.0.1:11434/api/tags &>/dev/null; then
        echo "ONLINE"
    else
        echo "OFFLINE"
    fi
    echo -n "2. Voice Microservice (GPU): "
    if curl -s http://127.0.0.1:8765/health &>/dev/null; then
        HEALTH=$(curl -s http://127.0.0.1:8765/health)
        echo "ONLINE ($HEALTH)"
    else
        echo "OFFLINE"
    fi
    exit 0
fi

MOCK_MODE=false
if [ "$1" == "--mock" ]; then
    MOCK_MODE=true
fi

echo "=================================================="
echo " 🎩 A.L.F.R.E.D. Master Launcher"
echo " Hardware: 4GB VRAM Hybrid Profile Active"
echo " - Ollama LLM  -> Hybrid (18 Layers on GPU VRAM, 19 on CPU)"
echo " - Whisper STT -> CPU (Instant transcription, 0MB VRAM)"
echo " - Coqui XTTS  -> NVIDIA GPU (CUDA:0, Memory-Optimized)"
echo "=================================================="

VOICE_PID=""
OLLAMA_PID=""

# Graceful cleanup handler
cleanup() {
    echo ""
    echo "[*] Shutting down A.L.F.R.E.D. background services..."
    if [ -n "$VOICE_PID" ] && kill -0 "$VOICE_PID" 2>/dev/null; then
        echo "[+] Terminating Voice Service (PID: $VOICE_PID)..."
        kill "$VOICE_PID" 2>/dev/null || true
    fi
    # Also kill any orphan voice service on port 8765
    pkill -P $$ 2>/dev/null || true
    echo "[+] Cleanup complete. Have a pleasant evening, Sir."
}
trap cleanup EXIT INT TERM

# ------------------------------------------------------------------------------
# 1. Start Ollama in Background
# ------------------------------------------------------------------------------
if ! curl -s http://127.0.0.1:11434/api/tags &>/dev/null; then
    echo "[+] Starting Ollama daemon..."
    ollama serve &> "$OLLAMA_LOG" &
    OLLAMA_PID=$!
    echo "[+] Ollama started in background (PID: $OLLAMA_PID). Waiting for ready..."
    
    READY=false
    for i in {1..15}; do
        if curl -s http://127.0.0.1:11434/api/tags &>/dev/null; then
            READY=true
            break
        fi
        sleep 0.5
    done
    
    if [ "$READY" = true ]; then
        echo "[+] Ollama is online on http://127.0.0.1:11434"
    else
        echo "[!] Warning: Ollama did not respond within 7 seconds. Continuing anyway..."
    fi
else
    echo "[+] Ollama daemon is already active on http://127.0.0.1:11434"
fi

# ------------------------------------------------------------------------------
# 2. Start Voice Microservice in Background (Whisper & XTTS on CUDA)
# ------------------------------------------------------------------------------
export COQUI_TOS_AGREED=1
if ! curl -s http://127.0.0.1:8765/health &>/dev/null; then
    echo "[+] Starting Voice Microservice in background..."
    
    MOCK_FLAG=""
    if [ "$MOCK_MODE" = true ]; then
        MOCK_FLAG="--mock"
        echo "[*] Voice service running in MOCK mode (no neural model weights needed)."
    fi

    bash "$PROJECT_ROOT/scripts/run_voice_service.sh" $MOCK_FLAG &> "$VOICE_LOG" &
    VOICE_PID=$!
    echo "[+] Voice service started (PID: $VOICE_PID, Log: $VOICE_LOG). Waiting for ready..."

    VOICE_READY=false
    for i in {1..20}; do
        if curl -s http://127.0.0.1:8765/health &>/dev/null; then
            VOICE_READY=true
            break
        fi
        sleep 0.5
    done

    if [ "$VOICE_READY" = true ]; then
        echo "[+] Voice Microservice is online on http://127.0.0.1:8765"
    else
        echo "[!] Voice service is still loading or running in background. Check $VOICE_LOG."
    fi
else
    echo "[+] Voice Microservice is already active on http://127.0.0.1:8765"
fi

# ------------------------------------------------------------------------------
# 3. Launch the Rust Ratatui Terminal UI
# ------------------------------------------------------------------------------
echo "=================================================="
echo "[+] All background services launched. Starting TUI..."
echo "=================================================="
sleep 1

if [ -f "$PROJECT_ROOT/target/release/alfred" ]; then
    exec "$PROJECT_ROOT/target/release/alfred"
else
    echo "[*] Precompiled binary not found. Building and running with cargo..."
    exec cargo run --release
fi
