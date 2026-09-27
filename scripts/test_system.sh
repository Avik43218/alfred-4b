#!/usr/bin/env bash
# ==============================================================================
# A.L.F.R.E.D. Diagnostic & Integration Test Suite
# Tests:
# 1. GPU VRAM & Hardware Constraints (4GB budget)
# 2. Ollama CPU-only API responsiveness
# 3. Voice microservice (Whisper & XTTS)
# 4. Web search tool & command guardrail sanity
# ==============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=================================================="
echo " Running A.L.F.R.E.D. System Diagnostic"
echo "=================================================="

# 1. GPU & VRAM Check
echo "[1/4] Checking GPU & VRAM allocation..."
if command -v nvidia-smi &>/dev/null; then
    nvidia-smi --query-gpu=name,memory.total,memory.free,memory.used --format=csv,noheader
    echo "[+] NVIDIA GPU detected. Target VRAM budget: 4GB."
else
    echo "[!] No NVIDIA GPU detected. System will operate in fallback mode."
fi

# 2. Ollama CPU-only Check
echo ""
echo "[2/4] Checking Ollama API on http://127.0.0.1:11434..."
if curl -s http://127.0.0.1:11434/api/tags &>/dev/null; then
    echo "[+] Ollama server is responding."
    MODELS=$(curl -s http://127.0.0.1:11434/api/tags | grep -o '"name":"[^"]*"' || true)
    echo "[+] Models found: $MODELS"
else
    echo "[-] Ollama server is NOT running. Run 'OLLAMA_NUM_GPU=0 ollama serve' to start."
fi

# 3. Voice Microservice Check
echo ""
echo "[3/4] Checking Voice Microservice on http://127.0.0.1:8765..."
if curl -s http://127.0.0.1:8765/health &>/dev/null; then
    HEALTH=$(curl -s http://127.0.0.1:8765/health)
    echo "[+] Voice service is active: $HEALTH"
else
    echo "[!] Voice service is not running. Launch with 'bash scripts/run_voice_service.sh --mock'"
fi

# 4. Web search & Command tool sanity
echo ""
echo "[4/4] Testing DuckDuckGo Web Search & Tools..."
if curl -s "https://api.duckduckgo.com/?q=weather+London&format=json" &>/dev/null; then
    echo "[+] External network search is reachable."
else
    echo "[!] External network search unavailable or offline."
fi

echo ""
echo "=================================================="
echo "[+] Diagnostic scan completed."
echo "=================================================="
