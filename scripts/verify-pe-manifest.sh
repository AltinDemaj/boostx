#!/usr/bin/env bash
set -euo pipefail
EXE="${1:-}"
if [[ -z "$EXE" ]]; then echo "usage: $0 <exe>" >&2; exit 1; fi
if [[ ! -f "$EXE" ]]; then echo "missing: $EXE" >&2; exit 1; fi
# stub - extend with real PE manifest checks
exit 0
