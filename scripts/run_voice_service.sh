#!/usr/bin/env bash
# ==============================================================================
# Launch A.L.F.R.E.D. Voice Microservice (Whisper STT + Coqui XTTS on CUDA)
# ==============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VENV_DIR="$PROJECT_ROOT/.venv"

if [ -d "$VENV_DIR" ]; then
    source "$VENV_DIR/bin/activate"
fi

# Automatically accept Coqui CPML non-commercial Terms of Service
export COQUI_TOS_AGREED=1
# Prevent PyTorch CUDA memory fragmentation on 4GB VRAM GPU
export PYTORCH_CUDA_ALLOC_CONF=expandable_segments:True

cd "$PROJECT_ROOT/voice_service"

MOCK_ARG=""
if [ "$1" == "--mock" ]; then
    MOCK_ARG="--mock"
    echo "[*] Launching voice service in MOCK/Simulated mode..."
else
    # Check if torch and cuda are available
    if ! python3 -c "import torch; exit(0 if torch.cuda.is_available() else 1)" 2>/dev/null; then
        echo "[!] Warning: CUDA is not detected or PyTorch is not installed in environment."
        echo "[!] Starting voice service in --mock mode for testing."
        MOCK_ARG="--mock"
    else
        echo "[+] CUDA detected! Launching XTTS on GPU (CUDA:0) and Whisper STT on CPU..."
    fi
fi

WHISPER_DEV="${WHISPER_DEVICE:-cpu}"

python3 service.py \
    --host 127.0.0.1 \
    --port 8765 \
    --whisper-device "$WHISPER_DEV" \
    --xtts-device cuda \
    $MOCK_ARG
