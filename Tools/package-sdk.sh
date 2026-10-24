#!/usr/bin/env bash
# 独立构建开发 SDK，跨平台收集规则由同一 Python 实现维护。
set -euo pipefail
python3 "$(dirname "$0")/package-sdk.py" "$@"
