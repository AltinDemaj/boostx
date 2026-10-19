# BoostX

**BoostX** is a Windows desktop optimizer for competitive gaming (CS2, VALORANT, Fortnite, and similar titles). It scans your hardware, applies safe Windows tweaks with full backup/revert, surfaces per-game guidance, and walks you through BIOS settings—without ever writing firmware.

Built with **Tauri 2** (Rust backend) and **React + TypeScript** (dark gaming UI). **Current release: v0.5.0**

## Features

| Area | Status |
|------|--------|
| Hardware scan (CPU/GPU/RAM/storage/board/BIOS/monitor/Windows) + performance score | Implemented (WMI on Windows; demo data elsewhere) |
| Hone-style tweak catalog with detect / apply / verify / revert | Implemented (powercfg GUID, netsh, registry inference) |
| Performance tab: game affinity watcher, timer service, NVIDIA power boost + temp guard | Implemented (AMD GPU/fans: guidance) |
| **Auto Optimize** — one-click baseline → per-tweak A/B benchmarks → keep/revert → report card | Implemented (runs locally; CS2 live when installed) |
| CS2 competitive benchmark (PresentMon live + labeled micro-bench fallback) | Implemented |
| Live benchmark charts (FPS, frametime spikes, CPU/GPU via nvidia-smi when available) | Implemented |
| System Restore point + per-value registry backup | Implemented |
| Steam library detection + CS2 launch options / autoexec / video hints | Implemented |
| BIOS checklist by vendor + reboot to firmware (`shutdown /r /fw`) | Implemented (read-only guidance) |
| Admin elevation manifest | Implemented (`requireAdministrator`) |
| Windows NSIS installer (cross-compile via cargo-xwin) | `npm run build:installer` |

### Stubbed / limited

- **RAM XMP/EXPO detection** — WMI rarely exposes rated XMP speed; UI flags this when data is missing (BIOS checklist covers enabling XMP/EXPO).
- **Valorant / Epic / Riot installs** — not auto-detected yet (Steam-focused); extend `src-tauri/src/games/`.
- **Startup app manager UI** — tweak list includes safe service/visual tweaks; dedicated startup UI not built.
- **In-game FPS benchmark** — use the built-in quick benchmark plus your own game FPS counter for validation.
- **Non-Windows hosts** — app runs for UI development with **demo scan data**; tweaks are no-ops.

## Name note

“BoostX” is the working title. Alternatives if you productize: **FrameForge**, **LatencyLab**, **CompPulse**.

## Prerequisites

### Windows (target)

- [Microsoft Edge WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (usually preinstalled on Windows 11)
- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 20+
- Visual Studio Build Tools with the **Desktop development with C++** workload (for MSVC linker)

### Linux (UI / Rust dev only)

Install [Tauri prerequisites](https://tauri.app/start/prerequisites/) if you want `tauri dev` locally. Windows installers are produced on Windows CI or a Windows machine.

## Development

```bash
npm install
npm run tauri dev
```

The app requests **administrator** elevation via its manifest. Accept the UAC prompt when testing tweaks.

## Production build (Windows)

```bash
npm install
npm run tauri build
```

Installers are written under:

- `src-tauri/target/release/bundle/msi/`
- `src-tauri/target/release/bundle/nsis/`

## Project layout

```
src/                 React UI
src-tauri/src/
  hardware/          WMI scan + registry inference)
  backup/            Restore points + registry backup manifest
  tweaks/            Tweak definitions + apply/revert engine
  games/             Steam detection + CS2 planner
  bios/              Vendor-specific BIOS checklists
  benchmark/         Quick before/after metrics
```

Backup data is stored under `%LOCALAPPDATA%\BoostX\` on Windows.

## Safety

1. A **System Restore** point is attempted before profile application (skipped if restore is disabled by policy).
2. Each registry tweak backs up the prior value (or “missing”) into `backups.json`.
3. Use **Revert** on a tweak or **Revert all** to restore backed-up values.

Always review BIOS checklist items before changing firmware settings.

## Landing page & download

The public site lives in [`landing/`](landing/). Deploy on Vercel with **Root Directory** = `landing`.

Direct download: `/download/BoostX-Setup.exe` (v0.1.0, ~2.1 MB).

Rebuild the installer on Linux (NSIS + `cargo-xwin`):

```bash
npm run build:installer
```

## CI

`.github/workflows/build-windows.yml` builds the NSIS installer on `windows-latest` and uploads artifacts (including the staged landing binary).

## License

MIT (add your own license file if you ship commercially.)
