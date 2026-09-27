import argparse
import os
import sys
import time

try:
    from fastapi import FastAPI, HTTPException, UploadFile, File
    from fastapi.responses import FileResponse
    from pydantic import BaseModel
    import uvicorn
    USE_FASTAPI = True
except ImportError:
    USE_FASTAPI = False
    from http.server import HTTPServer, BaseHTTPRequestHandler
    import json
    import urllib.parse

from stt_whisper import WhisperSTTEngine
from tts_xtts import CoquiXTTSEngine

# Global model engine singletons
stt_engine = None
tts_engine = None
is_mock_mode = False

def get_vram_usage():
    try:
        import torch
        if torch.cuda.is_available():
            allocated_mb = torch.cuda.memory_allocated() / (1024 * 1024)
            reserved_mb = torch.cuda.memory_reserved() / (1024 * 1024)
            return float(allocated_mb), float(reserved_mb)
    except Exception:
        pass
    return 0.0, 0.0

def get_health_data():
    allocated_mb, reserved_mb = get_vram_usage()
    return {
        "status": "healthy",
        "whisper_loaded": stt_engine is not None and not stt_engine.mock,
        "whisper_device": getattr(stt_engine, "device", "unknown"),
        "xtts_loaded": tts_engine is not None and not tts_engine.mock,
        "xtts_device": getattr(tts_engine, "device", "unknown"),
        "vram_allocated_mb": allocated_mb,
        "vram_reserved_mb": reserved_mb,
        "is_mock": is_mock_mode or (stt_engine is not None and stt_engine.mock),
    }

if USE_FASTAPI:
    app = FastAPI(title="A.L.F.R.E.D. Voice Microservice (Whisper STT + XTTS GPU)")

    class ListenRequest(BaseModel):
        duration_sec: int = 5
        language: str = "en"

    class SpeakRequest(BaseModel):
        text: str
        play_audio: bool = True
        language: str = "en"

    @app.get("/health")
    def health():
        return get_health_data()

    @app.post("/stt/listen")
    def listen_and_transcribe(req: ListenRequest):
        if stt_engine is None:
            raise HTTPException(status_code=503, detail="Whisper STT engine is not initialized")
        
        start = time.time()
        audio_data = stt_engine.record_microphone(duration_sec=req.duration_sec)
        text = stt_engine.transcribe(audio_data, language=req.language)
        elapsed = time.time() - start

        return {
            "text": text,
            "duration_sec": elapsed,
            "language": req.language
        }

    @app.post("/tts/speak")
    def speak(req: SpeakRequest):
        if tts_engine is None:
            raise HTTPException(status_code=503, detail="Coqui XTTS engine is not initialized")

        start = time.time()
        wav_path = tts_engine.synthesize(req.text, language=req.language)
        
        if req.play_audio:
            tts_engine.play_audio(wav_path)

        elapsed = time.time() - start
        return {
            "status": "success",
            "duration_sec": elapsed,
            "played": req.play_audio
        }

    @app.post("/tts/synthesize")
    def synthesize_wav(req: SpeakRequest):
        if tts_engine is None:
            raise HTTPException(status_code=503, detail="Coqui XTTS engine is not initialized")

        wav_path = tts_engine.synthesize(req.text, language=req.language)
        return FileResponse(wav_path, media_type="audio/wav", filename="alfred_speech.wav")

else:
    class AlfredFallbackHTTPHandler(BaseHTTPRequestHandler):
        def _send_json(self, status_code, data):
            payload = json.dumps(data).encode("utf-8")
            self.send_response(status_code)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def do_GET(self):
            parsed = urllib.parse.urlparse(self.path)
            if parsed.path == "/health":
                self._send_json(200, get_health_data())
            else:
                self._send_json(404, {"error": "Not Found"})

        def do_POST(self):
            parsed = urllib.parse.urlparse(self.path)
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length) if content_length > 0 else b"{}"

            try:
                data = json.loads(body.decode("utf-8")) if body else {}
            except Exception:
                data = {}

            if parsed.path == "/stt/listen":
                duration_sec = data.get("duration_sec", 5)
                language = data.get("language", "en")
                start = time.time()
                audio_data = stt_engine.record_microphone(duration_sec=duration_sec)
                text = stt_engine.transcribe(audio_data, language=language)
                elapsed = time.time() - start
                self._send_json(200, {
                    "text": text,
                    "duration_sec": elapsed,
                    "language": language
                })
            elif parsed.path == "/tts/speak":
                text = data.get("text", "")
                play_audio = data.get("play_audio", True)
                language = data.get("language", "en")
                start = time.time()
                wav_path = tts_engine.synthesize(text, language=language)
                if play_audio:
                    tts_engine.play_audio(wav_path)
                elapsed = time.time() - start
                self._send_json(200, {
                    "status": "success",
                    "duration_sec": elapsed,
                    "played": play_audio
                })
            else:
                self._send_json(404, {"error": "Endpoint not found"})

        def log_message(self, format, *args):
            # Suppress noisy standard request logs
            pass

def main():
    global stt_engine, tts_engine, is_mock_mode

    parser = argparse.ArgumentParser(description="A.L.F.R.E.D. Voice Microservice")
    parser.add_argument("--host", default="127.0.0.1", help="Host bind address")
    parser.add_argument("--port", type=int, default=8765, help="Port to listen on")
    parser.add_argument("--whisper-model", default="base.en", help="Whisper model size (base.en, small.en)")
    parser.add_argument("--whisper-device", default="cuda", help="Whisper device (cuda/cpu)")
    parser.add_argument("--xtts-device", default="cuda", help="XTTS device (cuda/cpu)")
    parser.add_argument("--speaker-wav", default=None, help="Reference speaker WAV for XTTS voice cloning")
    parser.add_argument("--mock", action="store_true", help="Run in mock/simulation mode without loading full weights")

    args = parser.parse_args()
    is_mock_mode = args.mock

    print("==================================================")
    print(" A.L.F.R.E.D. Voice Microservice")
    print(f" Server Engine      : {'FastAPI/Uvicorn' if USE_FASTAPI else 'Built-in Python HTTP Server'}")
    print(f" Whisper STT device : {args.whisper_device}")
    print(f" Coqui XTTS device  : {args.xtts_device}")
    print(f" Mock Mode Enabled  : {args.mock}")
    print(" (Ollama LLM is restricted to CPU system RAM)")
    print("==================================================")

    stt_engine = WhisperSTTEngine(
        model_size=args.whisper_model,
        device=args.whisper_device,
        compute_type="float16" if args.whisper_device == "cuda" else "int8",
        mock=args.mock
    )

    speaker_wav = args.speaker_wav
    if not speaker_wav:
        default_sample = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "assets", "voices", "alfred.wav"))
        if os.path.exists(default_sample):
            speaker_wav = default_sample
            print(f"[TTS] Detected custom voice profile at: {speaker_wav}")

    tts_engine = CoquiXTTSEngine(
        device=args.xtts_device,
        mock=args.mock,
        speaker_wav=speaker_wav
    )

    if USE_FASTAPI:
        uvicorn.run(app, host=args.host, port=args.port, log_level="info")
    else:
        server = HTTPServer((args.host, args.port), AlfredFallbackHTTPHandler)
        print(f"[+] Listening on http://{args.host}:{args.port}...")
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            print("\nShutting down voice service...")
            server.server_close()

if __name__ == "__main__":
    main()
