import customtkinter as ctk
import subprocess
import threading
import datetime
import pyaudio
import ollama
import random
import struct
import time
import wave
import math
import re
import os

try:
    import nvidia.cublas.lib
    import nvidia.cudnn.lib
    
    cublas_dir = list(nvidia.cublas.lib.__path__)[0]
    cudnn_dir = list(nvidia.cudnn.lib.__path__)[0]
    
    os.environ['LD_LIBRARY_PATH'] = f"{cublas_dir}:{cudnn_dir}:" + os.environ.get('LD_LIBRARY_PATH', '')
except Exception as e:
    print(f"[ CUDA PATH INJECT FAILED: {e} ]")

from faster_whisper import WhisperModel
from ddgs import DDGS

COMMAND_DECK = {
    "browser": "brave-browser",
    "terminal": "konsole",
    "files": "dolphin",
    "calculator": "kcalc"
}

def get_rms(data):
    # Calculates the volume (Root Mean Square) of the audio chunk
    count = len(data) // 2
    shorts = struct.unpack(f"{count}h", data)
    sum_squares = sum(s**2 for s in shorts)
    return math.sqrt(sum_squares / count) if count > 0 else 0

MIC_THRESHOLD = 800  
SILENCE_LIMIT = 2

ctk.set_appearance_mode("dark")

app = ctk.CTk()
app.title("A.L.F.R.E.D. Mainframe")
app.attributes('-fullscreen', True)
app.configure(fg_color="#030303")
app.bind("<Escape>", lambda e: app.destroy())

title_font = ("Consolas", 18, "bold")
hud_font = ("Consolas", 12)
chat_font = ("Consolas", 15)

hud_frame = ctk.CTkFrame(app, fg_color="transparent", corner_radius=0)
hud_frame.pack(fill="x", padx=40, pady=(20, 0))

date_str = datetime.datetime.now().strftime("LOCAL TIME: %H:%M:%S | %Y-%m-%d")
ctk.CTkLabel(hud_frame, text="VICTUS MAINFRAME | GHOST", text_color="#00ffff", font=title_font).pack(side="left")
ctk.CTkLabel(hud_frame, text=f"UPLINK SECURE | {date_str}", text_color="#007777", font=hud_font).pack(side="right")

chat_log = ctk.CTkTextbox(
    app, fg_color="#080808", text_color="#00ffff", font=chat_font, 
    corner_radius=0, border_width=1, border_color="#004444"
)
chat_log.pack(fill="both", expand=True, padx=40, pady=20)
chat_log.configure(state="disabled")

input_frame = ctk.CTkFrame(app, fg_color="transparent")
input_frame.pack(fill="x", padx=40, pady=(0, 40))

status_label = ctk.CTkLabel(input_frame, text="[ BOOTING SYSTEMS ]", text_color="#00ffff", font=("Consolas", 24, "bold"))
status_label.pack(pady=10)

def update_status(text, color):
    status_label.configure(text=text, text_color=color)

def append_chat(speaker, text):
    chat_log.configure(state="normal")
    if speaker == "USER":
        chat_log.insert("end", f"> USER INPUT: {text.title()}\n", "user")
    else:
        chat_log.insert("end", f"A.L.F.R.E.D: {text}\n\n", "ai")
    
    chat_log.configure(state="disabled")
    chat_log.yview("end")

def jarvis_speak(text):
    subprocess.run(
        ["edge-playback", "--voice", "en-US-AndrewNeural", "--text", text],
        stderr=subprocess.DEVNULL
    )

def run_jarvis():
    app.after(0, update_status, "[ BOOTING... ]", "#ff00ff")
    whisper_model = WhisperModel("base.en", device="cuda", compute_type="float16")
    
    boot_sequence = [
        "INITIALIZING CORE KERNEL ENCLAVE...",
        "BYPASSING SECURITY PROTOCOL 7A...",
        "MOUNTING VIRTUAL FILE SYSTEMS [ OK ]",
        "DECRYPTING NEURAL WEIGHTS (0x00F3A92B)...",
        "LOADING FASTER-WHISPER AUDIO MATRICES...",
        "ESTABLISHING QUANTUM UPLINK...",
        "SYNCING SATELLITE TELEMETRY...",
        "ALL SYSTEMS NOMINAL."
    ]
    
    chat_log.configure(state="normal")
    for line in boot_sequence:
        chat_log.insert("end", f"[ SYSTEM ] {line}\n")
        chat_log.yview("end")
        time.sleep(random.uniform(0.1, 0.4))
    chat_log.insert("end", "="*80 + "\n\n")
    chat_log.configure(state="disabled")
    
    app.after(0, append_chat, "ALFRED", "Systems online. What's good, boss?")
    app.after(0, update_status, "[ SPEAKING ]", "#00ff00")
    jarvis_speak("Systems online. What's good, boss?")
    
    chat_history = []
    temp_audio_path = "temp_alfred_audio.wav"

    while True:
        try:
            app.after(0, update_status, "[ PASSIVE SCAN: AWAITING VOICE... ]", "#00ffff")
            
            pa = pyaudio.PyAudio()
            stream = pa.open(format=pyaudio.paInt16, channels=1, rate=16000, input=True, frames_per_buffer=1024)
            
            while True:
                data = stream.read(1024, exception_on_overflow=False)
                if get_rms(data) > MIC_THRESHOLD:
                    break
            
            app.after(0, update_status, "[ RECORDING... ]", "#ffaa00")
            
            frames = [data]
            silent_chunks = 0
            while True:
                data = stream.read(1024, exception_on_overflow=False)
                frames.append(data)
                
                if get_rms(data) < MIC_THRESHOLD:
                    silent_chunks += 1
                else:
                    silent_chunks = 0
                    
                if silent_chunks > int((16000 / 1024) * SILENCE_LIMIT):
                    break
                    
            stream.stop_stream()
            stream.close()
            pa.terminate()
            
            with wave.open(temp_audio_path, 'wb') as wf:
                wf.setnchannels(1)
                wf.setsampwidth(2)
                wf.setframerate(16000)
                wf.writeframes(b''.join(frames))
            
            app.after(0, update_status, "[ DECODING AUDIO (CUDA)... ]", "#ffaa00")
            
            segments, _ = whisper_model.transcribe(
                temp_audio_path, 
                beam_size=5,
                vad_filter=True, 
                vad_parameters=dict(min_silence_duration_ms=500),
                initial_prompt="Hello ALFRED, Boss, Victus Mainframe.",
                condition_on_previous_text=False
            )
            user_text = "".join([segment.text for segment in segments]).strip()
            
            if os.path.exists(temp_audio_path):
                os.remove(temp_audio_path)
            
            if len(user_text) < 2:
                continue

            app.after(0, append_chat, "USER", user_text)
            app.after(0, update_status, "[ NEURAL NET PROCESSING... ]", "#ff00ff")
            
            current_time = datetime.datetime.now().strftime("%A, %B %d, %Y - %I:%M %p")
            chat_history.append({'role': 'user', 'content': user_text})
            
            if len(chat_history) > 6:
                chat_history = chat_history[-6:]
            
            temp_history = chat_history.copy()
            temp_history[-1]['content'] = f"[System Clock: {current_time}] {user_text}"
            
            response = ollama.chat(
                model='alfred:latest',
                messages=temp_history, 
                stream=False
            )
            
            reply = response['message']['content']
            chat_history.append({'role': 'assistant', 'content': reply})
            
            commands_to_run = re.findall(r'\[EXECUTE:\s*(.*?)\]', reply, re.IGNORECASE)
            for cmd in commands_to_run:
                clean_cmd = cmd.lower().strip()
                if clean_cmd in COMMAND_DECK:
                    app.after(0, append_chat, "SYSTEM", f"Executing command: {COMMAND_DECK[clean_cmd]}")
                    subprocess.Popen(COMMAND_DECK[clean_cmd].split(), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            
            search_requests = re.findall(r'\[SEARCH:\s*(.*?)\]', reply, re.IGNORECASE)
            for search_query in search_requests:
                app.after(0, update_status, "[ QUANTUM UPLINK: SCRAPING WEB... ]", "#00ffff")
                try:
                    results = DDGS().text(search_query, max_results=3)
                    web_dump = f"LIVE UPLINK: '{search_query}'\n" + "-"*40 + "\n"
                    for res in results:
                        web_dump += f"{res['title']}\n {res['href']}\n\n"
                    app.after(0, append_chat, "SYSTEM", web_dump)
                except Exception as e:
                    app.after(0, append_chat, "SYSTEM", f"UPLINK FAILED: {e}")

            clean_reply = re.sub(r'\[EXECUTE:.*?\]', '', reply, flags=re.IGNORECASE)
            clean_reply = re.sub(r'\[SEARCH:.*?\]', '', clean_reply, flags=re.IGNORECASE)
            clean_reply = re.sub(r'<think>.*?</think>', '', clean_reply, flags=re.DOTALL)
            clean_reply = re.sub(r'\*.*?\*', '', clean_reply)
            clean_reply = re.sub(r'[\U00010000-\U0010ffff]', '', clean_reply).strip()

            if clean_reply:
                app.after(0, append_chat, "ALFRED", clean_reply)
                app.after(0, update_status, "[ BROADCASTING... ]", "#00ff00")
                jarvis_speak(clean_reply)
            
        except Exception as e:
            app.after(0, append_chat, "SYSTEM", f"RUNTIME ERROR: {e}")

threading.Thread(target=run_jarvis, daemon=True).start()

app.mainloop()
