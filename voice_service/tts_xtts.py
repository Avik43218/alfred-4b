import os
import subprocess
import tempfile
import time
import math
import wave
import struct

# 1. Automatically agree to Coqui CPML non-commercial Terms of Service
os.environ["COQUI_TOS_AGREED"] = "1"

# 2. Ensure TTS_HOME cache directory is writable (checks ~/.local/share/tts or falls back to workspace)
if "TTS_HOME" not in os.environ:
    local_tts = os.path.expanduser("~/.local/share/tts")
    try:
        os.makedirs(local_tts, exist_ok=True)
    except Exception:
        workspace_tts = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".models", "tts"))
        os.makedirs(workspace_tts, exist_ok=True)
        os.environ["TTS_HOME"] = workspace_tts

# 3. Compatibility shim for newer transformers versions where isin_mps_friendly was removed
try:
    import torch
    import transformers.pytorch_utils
    if not hasattr(transformers.pytorch_utils, "isin_mps_friendly"):
        def isin_mps_friendly(elements, test_elements):
            return torch.isin(elements, test_elements)
        transformers.pytorch_utils.isin_mps_friendly = isin_mps_friendly
except Exception as e:
    pass

class CoquiXTTSEngine:
    def __init__(self, model_name="tts_models/multilingual/multi-dataset/xtts_v2", device="cuda", mock=False, speaker_wav=None):
        self.model_name = model_name
        self.device = device
        self.mock = mock
        self.speaker_wav = speaker_wav
        self.tts = None

        if self.mock:
            print(f"[TTS] Running in MOCK mode (XTTS simulated on {self.device})")
            return

        try:
            import torch
            if device == "cuda" and not torch.cuda.is_available():
                print("[TTS WARNING] CUDA requested but not available. Falling back to CPU.")
                self.device = "cpu"

            from TTS.api import TTS
            print(f"[TTS] Loading Coqui XTTS v2 onto {self.device}... (this may take a few moments on initial download)")
            start = time.time()
            self.tts = TTS(model_name=self.model_name).to(self.device)
            if self.device == "cuda":
                torch.cuda.empty_cache()
            print(f"[TTS] Coqui XTTS loaded successfully in {time.time() - start:.2f}s on {self.device}!")
        except Exception as e:
            print(f"[TTS ERROR] Failed to load Coqui XTTS on {self.device}: {e}")
            print("[TTS] Falling back to simulated/system TTS mode.")
            self.mock = True

    def synthesize(self, text, output_path=None, language="en"):
        """Synthesizes text into speech WAV file using Coqui XTTS v2."""
        if not output_path:
            tmp = tempfile.NamedTemporaryFile(suffix=".wav", delete=False)
            output_path = tmp.name
            tmp.close()

        if self.mock or self.tts is None:
            # If model is not loaded, create an audible notification chime rather than silence
            self._create_audible_tone(output_path)
            return output_path

        try:
            # If speaker_wav reference is provided, clone speaker voice;
            # Otherwise use built-in speaker (Damian Black is a deep, cultured British butler tone)
            if self.speaker_wav and os.path.exists(self.speaker_wav):
                self.tts.tts_to_file(
                    text=text,
                    speaker_wav=self.speaker_wav,
                    language=language,
                    file_path=output_path
                )
            else:
                available_speakers = getattr(self.tts, "speakers", [])
                speaker_choice = "Damian Black" if "Damian Black" in available_speakers else (available_speakers[0] if available_speakers else None)
                if speaker_choice:
                    self.tts.tts_to_file(
                        text=text,
                        speaker=speaker_choice,
                        language=language,
                        file_path=output_path
                    )
                else:
                    self.tts.tts_to_file(
                        text=text,
                        language=language,
                        file_path=output_path
                    )
            return output_path
        except Exception as e:
            print(f"[TTS ERROR] XTTS synthesis failed: {e}")
            self._create_audible_tone(output_path)
            return output_path
        finally:
            if self.device == "cuda":
                try:
                    import torch
                    torch.cuda.empty_cache()
                except Exception:
                    pass

    def play_audio(self, wav_path):
        """Plays the synthesized audio file over system speakers."""
        if not os.path.exists(wav_path):
            return

        # 1. PipeWire native player
        try:
            subprocess.run(["pw-play", wav_path], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            return
        except Exception:
            pass

        # 2. PulseAudio / ALSA compatibility
        for player_cmd in [["paplay", wav_path], ["aplay", "-q", wav_path], ["pw-cat", "-p", wav_path]]:
            try:
                subprocess.run(player_cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                return
            except Exception:
                continue

        # 3. Python sounddevice fallback
        try:
            import sounddevice as sd
            from scipy.io import wavfile
            sr, data = wavfile.read(wav_path)
            sd.play(data, sr)
            sd.wait()
        except Exception as e:
            print(f"[TTS] Audio playback fallback failed: {e}")

    def _create_audible_tone(self, file_path):
        """Generates a pleasant double chime (440Hz and 880Hz) so the user knows audio is working even before neural weights finish."""
        try:
            with wave.open(file_path, "w") as wav_file:
                wav_file.setnchannels(1)
                wav_file.setsampwidth(2)
                wav_file.setframerate(16000)
                # Tone 1: 440 Hz for 0.15s
                for i in range(2400):
                    val = int(8000 * math.sin(2 * math.pi * 440 * i / 16000))
                    wav_file.writeframes(struct.pack('<h', val))
                # Tone 2: 880 Hz for 0.25s
                for i in range(4000):
                    val = int(10000 * math.sin(2 * math.pi * 880 * i / 16000))
                    wav_file.writeframes(struct.pack('<h', val))
        except Exception as e:
            print(f"[TTS] Failed to create notification tone: {e}")
