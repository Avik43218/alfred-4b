#!/usr/bin/env bash
# ==============================================================================
# A.L.F.R.E.D. Custom Voice Cloner & Recorder
# Records or imports reference audio for zero-shot voice cloning in Coqui XTTS v2
# ==============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VOICE_DIR="$PROJECT_ROOT/assets/voices"
OUTPUT_WAV="$VOICE_DIR/alfred.wav"
VENV_DIR="$PROJECT_ROOT/.venv"

mkdir -p "$VOICE_DIR"

if [ -d "$VENV_DIR" ]; then
    source "$VENV_DIR/bin/activate"
fi

echo "=================================================="
echo " 🎙️ A.L.F.R.E.D. Custom Voice Setup"
echo "=================================================="

if [ -n "$1" ]; then
    INPUT_FILE="$1"
    if [ ! -f "$INPUT_FILE" ]; then
        echo "[-] Error: Specified audio file does not exist: $INPUT_FILE"
        exit 1
    fi
    echo "[+] Importing reference audio from: $INPUT_FILE"
    echo "[+] Converting to 22.05kHz 16-bit mono WAV..."
    ffmpeg -y -i "$INPUT_FILE" -ac 1 -ar 22050 -sample_fmt s16 "$OUTPUT_WAV" -loglevel error
    echo "[+] Successfully saved custom voice profile to:"
    echo "    $OUTPUT_WAV"
else
    echo "This tool records a 6-second clean audio sample from your microphone."
    echo "Tip: Speak clearly, without background noise, in the tone you want Alfred to use."
    echo ""
    echo "Sample sentence to read aloud:"
    echo '  "Good day, Sir. I am Alfred, your faithful digital assistant. How may I be of service today?"'
    echo ""
    read -p "Press [Enter] when ready to start recording for 6 seconds..."
    echo ""
    echo "[🔴 RECORDING NOW... Speak into your microphone!]"

    # Record 6 seconds at 22050 Hz 16-bit mono
    if command -v arecord &>/dev/null; then
        arecord -d 6 -r 22050 -f S16_LE -c 1 "$OUTPUT_WAV" 2>/dev/null || \
        ffmpeg -y -f pulse -i default -t 6 -ar 22050 -ac 1 "$OUTPUT_WAV" -loglevel error
    else
        ffmpeg -y -f pulse -i default -t 6 -ar 22050 -ac 1 "$OUTPUT_WAV" -loglevel error
    fi

    echo "[+] Recording complete! Saved to:"
    echo "    $OUTPUT_WAV"
fi

echo ""
echo "[*] Testing playback of your recorded sample..."
for player in [['pw-play', "$OUTPUT_WAV"], ['paplay', "$OUTPUT_WAV"], ['aplay', '-q', "$OUTPUT_WAV"]]; do
    if command -v "${player[0]}" &>/dev/null; then
        "${player[0]}" "$OUTPUT_WAV" 2>/dev/null && break || true
    fi
done

echo ""
echo "=================================================="
echo "[+] Voice profile is configured!"
echo "    The Voice service will now automatically clone this voice."
echo "    Restart Alfred with: ./start-alfred.sh"
echo "=================================================="
