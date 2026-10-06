#!/bin/sh
# Entry point for the ship gate. The commands live in verify.ps1 because
# Git Bash does not put dotnet on its own PATH.
set -eu
cd "$(dirname "$0")"
if command -v pwsh.exe >/dev/null 2>&1; then
  exec pwsh.exe -NoProfile -File ./verify.ps1
fi
if command -v pwsh >/dev/null 2>&1; then
  exec pwsh -NoProfile -File ./verify.ps1
fi
exec powershell.exe -NoProfile -File ./verify.ps1
