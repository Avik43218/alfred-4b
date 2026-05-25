#!/bin/bash

CUBLAS_PATH="/usr/local/lib/ollama/cuda_v12/"
CUDNN_PATH=$(python3 -c "import nvidia.cudnn.lib as lib; print(list(lib.__path__)[0])" 2>/dev/null)

export LD_LIBRARY_PATH="$CUBLAS_PATH:$LD_LIBRARY_PATH"

echo "[ LAUNCHING VICTUS MAINFRAME... ]"
python3 source/alfred.py
