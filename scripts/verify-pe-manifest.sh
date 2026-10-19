#!/usr/bin/env bash
set -euo pipefail
EXE="${1:-src-tauri/target/x86_64-pc-windows-msvc/release/boostx.exe}"
if [[ ! -f "$EXE" ]]; then
  echo "boostx.exe not found at $EXE"
  exit 1
fi
if grep -a "requireAdministrator" "$EXE" >/dev/null 2>&1; then
  echo "OK: PE embeds requireAdministrator manifest ($EXE)"
else
  echo "WARN: requireAdministrator not found in $EXE (check winres / app.manifest)"
  exit 1
fi
