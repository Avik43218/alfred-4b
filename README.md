# ALFRED-4B
A.L.F.R.E.D. is a personal AI assistant, powered by qwen3:4b-instruct model, with voice chat capabilities, web scraping and command execution.

It runs locally via Ollama, for a complete offline yet premium-tier experience.

---

## Setup Guide

#### 1. Create a virtual environment
```bash
python -m venv .venv
source .venv/bin/activate
```

#### 2. Install pre-requisites
```bash
pip install -r requirements.txt
```
(Also make sure that `ollama` server is installed on your system. Then install the `qwen3:4b-instruct` model. You may also need to install the `nvidia.cublas` library, if not already.)

#### 3. Clone the repository
```bash
git clone https://github.com/Avik43218/alfred-4b/
cd alfred-4b/
```

#### 4. Start chatting
```bash
./boot.sh
```

---

## License

This project is maintained under the [MIT LICENSE](LICENSE).
