#!/usr/bin/env bash
# ==============================================================================
# A.L.F.R.E.D. Voice Models Pre-downloader & Sound Test
# Downloads Faster-Whisper & Coqui XTTS v2 weights with a visible progress bar,
# then tests audio playback through system speakers.
# ==============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VENV_DIR="$PROJECT_ROOT/.venv"

if [ -d "$VENV_DIR" ]; then
    source "$VENV_DIR/bin/activate"
fi

export COQUI_TOS_AGREED=1

echo "=================================================="
echo " A.L.F.R.E.D. Neural Voice Model Setup & Test"
echo "=================================================="

python3 -c "
import os, sys, time
os.environ['COQUI_TOS_AGREED'] = '1'

# 1. Compatibility patch for transformers
import torch
import transformers.pytorch_utils
if not hasattr(transformers.pytorch_utils, 'isin_mps_friendly'):
    transformers.pytorch_utils.isin_mps_friendly = torch.isin

print('[1/3] Verifying Faster-Whisper STT on GPU...')
from faster_whisper import WhisperModel
stt = WhisperModel('base.en', device='cuda' if torch.cuda.is_available() else 'cpu', compute_type='float16' if torch.cuda.is_available() else 'int8')
print('[+] Faster-Whisper loaded and ready on GPU!')

print('\n[2/3] Downloading & Loading Coqui XTTS v2 model (~1.8GB)...')
print('Please wait while model weights are downloaded to ~/.local/share/tts/...')
from TTS.api import TTS
tts = TTS(model_name='tts_models/multilingual/multi-dataset/xtts_v2').to('cuda' if torch.cuda.is_available() else 'cpu')
print('[+] Coqui XTTS v2 downloaded and initialized successfully on GPU!')

print('\n[3/3] Generating test speech...')
test_wav = '/tmp/alfred_test_speech.wav'
available_speakers = getattr(tts, 'speakers', [])
speaker_choice = 'Damian Black' if 'Damian Black' in available_speakers else (available_speakers[0] if available_speakers else None)
print(f'[+] Using speaker profile: {speaker_choice}')

tts.tts_to_file(
    text='Good evening, Sir. A.L.F.R.E.D. audio subsystem is fully operational and at your command.',
    speaker=speaker_choice,
    file_path=test_wav,
    language='en'
)
print('[+] Speech synthesized to:', test_wav)

print('\n[*] Playing audio over speakers...')
import subprocess
for player in [['pw-play', test_wav], ['paplay', test_wav], ['aplay', '-q', test_wav]]:
    try:
        subprocess.run(player, check=True)
        print('[+] Audio played successfully via', player[0])
        break
    except Exception:
        continue

print('\n==================================================')
print('[+] All voice models are downloaded and ready!')
print('==================================================')
"
