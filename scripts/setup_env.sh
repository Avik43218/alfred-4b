#!/usr/bin/env bash
# ==============================================================================
# A.L.F.R.E.D. Python Environment Setup Script
# Configures Python virtual environment with PyTorch CUDA, Faster-Whisper, and Coqui XTTS
# ==============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VENV_DIR="$PROJECT_ROOT/.venv"

echo "=================================================="
echo " Setting up A.L.F.R.E.D. Voice Environment"
echo " Target: CUDA GPU for Whisper STT & Coqui XTTS"
echo " (Ollama LLM runs on CPU system RAM)"
echo "=================================================="

# Prefer Python 3.12, 3.11, or 3.10 (best compatibility with CUDA PyTorch and TTS)
PYTHON_BIN=""
for py in python3.12 python3.11 python3.10 python3; do
    if command -v "$py" &>/dev/null; then
        # Check if version is < 3.13 for best torch/TTS binary wheel support
        PY_VER=$("$py" -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')")
        PYTHON_BIN="$py"
        if [[ "$py" == "python3.12" || "$py" == "python3.11" || "$py" == "python3.10" ]]; then
            break
        fi
    fi
done

if [ -z "$PYTHON_BIN" ]; then
    echo "[-] Error: Python 3 not found on system."
    exit 1
fi

echo "[+] Using Python binary: $PYTHON_BIN ($($PYTHON_BIN --version))"

# If an old/incompatible venv exists, remove it
if [ -d "$VENV_DIR" ]; then
    VENV_PY_VER=$("$VENV_DIR/bin/python" -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')" 2>/dev/null || echo "0.0")
    if [[ "$VENV_PY_VER" == "3.14"* ]]; then
        echo "[!] Existing venv was created with Python 3.14 (incompatible with TTS wheels). Recreating with $PYTHON_BIN..."
        rm -rf "$VENV_DIR"
    fi
fi

if [ ! -d "$VENV_DIR" ]; then
    echo "[+] Creating virtual environment in $VENV_DIR using $PYTHON_BIN..."
    "$PYTHON_BIN" -m venv "$VENV_DIR"
fi

echo "[+] Activating virtual environment..."
source "$VENV_DIR/bin/activate"

echo "[+] Upgrading pip and build tools..."
pip install --upgrade pip setuptools wheel

echo "[+] Installing PyTorch with CUDA support..."
# Install torch with CUDA support
pip install torch torchaudio --index-url https://download.pytorch.org/whl/cu124 || pip install torch torchaudio

echo "[+] Installing Faster-Whisper, Coqui XTTS, FastAPI, and sound drivers..."
pip install -r "$PROJECT_ROOT/voice_service/requirements.txt" || pip install coqui-tts faster-whisper ctranslate2 fastapi "uvicorn[standard]" sounddevice scipy requests

echo "[+] Verifying GPU availability..."
python3 -c "import torch; print('CUDA Available:', torch.cuda.is_available()); print('Device Name:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'N/A')"

echo "=================================================="
echo "[+] Environment setup completed successfully!"
echo "    Activate with: source $VENV_DIR/bin/activate"
echo "=================================================="
