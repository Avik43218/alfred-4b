#!/usr/bin/env bash
# ==============================================================================
# A.L.F.R.E.D. Ollama CPU-Only Setup & Model Builder
# CRITICAL HARDWARE CONSTRAINT:
# OLLAMA_NUM_GPU=0 forces Ollama to run 100% on CPU/system RAM, leaving the 4GB
# VRAM entirely free for Whisper STT and Coqui XTTS on GPU.
# ==============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=================================================="
echo " Configuring Ollama for CPU-Only Execution"
echo " (Preserving all 4GB VRAM for Whisper & Coqui XTTS)"
echo "=================================================="

# Export CPU-only constraint
export OLLAMA_NUM_GPU=0

# Check if Ollama server is running
if ! curl -s http://127.0.0.1:11434/api/tags &>/dev/null; then
    echo "[+] Ollama server is not running. Starting Ollama with OLLAMA_NUM_GPU=0..."
    OLLAMA_NUM_GPU=0 ollama serve &
    OLLAMA_PID=$!
    echo "[+] Waiting for Ollama daemon to initialize (PID: $OLLAMA_PID)..."
    for i in {1..15}; do
        if curl -s http://127.0.0.1:11434/api/tags &>/dev/null; then
            echo "[+] Ollama is online and listening on http://127.0.0.1:11434!"
            break
        fi
        sleep 1
    done
else
    echo "[+] Ollama daemon is already active."
fi

# Pull base model or build alfred model
cd "$PROJECT_ROOT"

# Check if Modelfile exists
if [ -f "Modelfile" ]; then
    BASE_MODEL=$(grep "^FROM" Modelfile | awk '{print $2}')
    echo "[+] Base model in Modelfile is: $BASE_MODEL"

    echo "[+] Pulling base model '$BASE_MODEL' (or fallback qwen2.5:3b)..."
    if ! ollama pull "$BASE_MODEL"; then
        echo "[!] Primary base model '$BASE_MODEL' pull failed. Trying fallback 'qwen2.5:3b'..."
        ollama pull qwen2.5:3b
        sed -i 's/^FROM .*/FROM qwen2.5:3b/' Modelfile
    fi

    echo "[+] Building customized A.L.F.R.E.D. butler model from Modelfile..."
    ollama create alfred -f Modelfile
    echo "[+] Model 'alfred' created successfully!"
fi

echo "=================================================="
echo "[+] Ollama setup complete! Available models:"
ollama list
echo "=================================================="
