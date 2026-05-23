#!/bin/bash

CUBLAS_PATH=$(python3 -c "import nvidia.cublas.lib as lib; print(list(lib.__path__)[0])" 2>/dev/null)
CUDNN_PATH=$(python3 -c "import nvidia.cudnn.lib as lib; print(list(lib.__path__)[0])" 2>/dev/null)

export LD_LIBRARY_PATH="$CUBLAS_PATH:$CUDNN_PATH:$LD_LIBRARY_PATH"

echo "[ LAUNCHING VICTUS MAINFRAME... ]"
python3 source/jarvis.py
