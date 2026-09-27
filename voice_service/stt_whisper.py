import io
import os
import subprocess
import tempfile
import time
import numpy as np

class WhisperSTTEngine:
    def __init__(self, model_size="base.en", device="cuda", compute_type="float16", mock=False):
        self.model_size = model_size
        self.device = device
        self.compute_type = compute_type
        self.mock = mock
        self.model = None

        if self.mock:
            print(f"[STT] Running in MOCK mode (Whisper simulated on {self.device})")
            return

        try:
            import torch
            if device == "cuda" and not torch.cuda.is_available():
                print("[STT WARNING] CUDA requested but not available. Falling back to CPU for Whisper.")
                self.device = "cpu"
                self.compute_type = "int8"

            from faster_whisper import WhisperModel
            print(f"[STT] Loading faster-whisper '{model_size}' onto {self.device} ({self.compute_type})...")
            start = time.time()
            self.model = WhisperModel(
                model_size,
                device=self.device,
                compute_type=self.compute_type
            )
            print(f"[STT] Whisper loaded successfully in {time.time() - start:.2f}s on {self.device}!")
        except Exception as e:
            print(f"[STT ERROR] Failed to load faster-whisper on {self.device}: {e}")
            print("[STT] Falling back to simulated/mock STT mode.")
            self.mock = True

    def record_microphone(self, duration_sec=5, sample_rate=16000):
        """Records raw PCM audio from default microphone."""
        if self.mock:
            time.sleep(duration_sec)
            return np.zeros(int(duration_sec * sample_rate), dtype=np.float32)

        # 1. Try sounddevice
        try:
            import sounddevice as sd
            print(f"[STT] Recording {duration_sec}s from microphone via sounddevice...")
            audio = sd.rec(int(duration_sec * sample_rate), samplerate=sample_rate, channels=1, dtype='float32')
            sd.wait()
            return audio.flatten()
        except Exception as e:
            print(f"[STT] sounddevice capture unavailable ({e}), trying arecord / ffmpeg...")

        # 2. Try arecord
        try:
            with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as f:
                temp_wav = f.name
            
            cmd = ["arecord", "-d", str(int(duration_sec)), "-r", str(sample_rate), "-f", "S16_LE", "-c", "1", temp_wav]
            subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            
            from scipy.io import wavfile
            sr, data = wavfile.read(temp_wav)
            os.remove(temp_wav)
            if data.dtype == np.int16:
                return data.astype(np.float32) / 32768.0
            return data.astype(np.float32)
        except Exception as e:
            print(f"[STT] arecord fallback failed: {e}")

        # 3. Try pw-cat / ffmpeg
        try:
            with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as f:
                temp_wav = f.name
            cmd = ["ffmpeg", "-y", "-f", "pulse", "-i", "default", "-t", str(duration_sec), "-ar", str(sample_rate), "-ac", "1", temp_wav]
            subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            from scipy.io import wavfile
            sr, data = wavfile.read(temp_wav)
            os.remove(temp_wav)
            return data.astype(np.float32) / 32768.0
        except Exception as e:
            print(f"[STT] Microphone recording fallback failed: {e}")
            return np.zeros(int(duration_sec * sample_rate), dtype=np.float32)

    def transcribe(self, audio_data, language="en"):
        """Transcribes audio numpy array to text using faster-whisper."""
        if self.mock or self.model is None:
            # Simulated responses for quick verification without weights
            return "Alfred, check the system uptime and list directory files."

        try:
            segments, info = self.model.transcribe(
                audio_data,
                beam_size=5,
                language=language,
                vad_filter=True,
                vad_parameters=dict(min_silence_duration_ms=500)
            )
            text_parts = [segment.text for segment in segments]
            full_text = " ".join(text_parts).strip()
            return full_text
        except Exception as e:
            print(f"[STT ERROR] Transcription failed: {e}")
            return ""
