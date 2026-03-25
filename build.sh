#!/usr/bin/env bash
set -euo pipefail

if [ $# -eq 0 ]; then
  cargo build --release
else
  cargo zigbuild --release --target "$1"
fi
