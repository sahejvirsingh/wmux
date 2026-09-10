# wmux — cmux for Windows

A Windows-native terminal multiplexer inspired by [cmux](https://cmux.com) (macOS-only). Built for developers who run multiple terminals and AI coding agents side by side.

**Stack:** Tauri v2 (Rust) + React (TypeScript) + xterm.js (WebGL)

## Features

- **Split panes** — horizontal + vertical splits with drag-to-resize and keyboard navigation
- **Vertical tab sidebar** — workspaces with git branch, CWD, and active ports
- **Notification rings** — panes glow when AI agents need attention (OSC 9/99/777 parsing) + Windows toast notifications
- **Agent session resume** — auto-detects Claude Code, Codex, Gemini CLI, Antigravity, OpenCode; re-runs `--resume` commands
- **Session restore** — layout, scrollback, CWD, and browser state persisted (JSON snapshot + SQLite)
- **CLI + Named pipe API** — scriptable via `\\.\pipe\wmux`
- **SSH & remote tmux attach** — via the `ssh2` crate
- **In-app browser pane** — WebView2 (ships natively with Windows)
- **GPU-accelerated rendering** — xterm.js WebGL addon
- **Theme engine** — Midnight, Daylight, Nord, Dracula + custom themes
- **Windows polish** — Mica/Acrylic backdrop, custom titlebar, system tray

## Prerequisites

- [Node.js](https://nodejs.org/) (LTS)
- [Rust](https://rustup.rs/) (stable toolchain)
- Windows 10/11 (WebView2 is included with Windows 11; on Windows 10 install it from Microsoft)
- Tauri v2 system dependencies — see the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/)

## Getting Started

```powershell
# 1. Install frontend dependencies (first time only)
npm install

# 2. Run the app in dev mode
npm run tauri dev
```

This compiles the Rust backend and opens the wmux window with hot-reloading for the frontend.

### Production Build

```powershell
npm run tauri build
```

Produces a standalone installer + `.exe` in `src-tauri/target/release/bundle/`.

## Using the CLI

A standalone `wmux.exe` CLI communicates with the running app over the Windows named pipe `\\.\pipe\wmux`. **The wmux app must be running first.**

The repo ships a debug build at `cli\target\debug\wmux.exe`. To rebuild it:

```powershell
cd cli
cargo build
```

### Commands

```powershell
# Workspace commands
wmux list-workspaces
wmux new-workspace
wmux select-workspace --workspace <id>

# Split commands
wmux new-split right
wmux new-split down --command "npm run dev"
wmux open-browser https://example.com

# Surface commands
wmux send-text "hello world"
wmux read-screen --surface <id>

# Agent hooks
wmux hooks setup
wmux hooks setup claude

# Session
wmux restore-session
```

All commands support `--json` for machine-readable output:

```powershell
wmux list-workspaces --json
```

Protocol: JSON-RPC style over the named pipe:

```json
{"id": "req-1", "method": "workspace.list", "params": {}}
→ {"id": "req-1", "ok": true, "result": {"workspaces": [...]}}
```

## Keyboard Shortcuts

| Action | Shortcut |
|---|---|
| Command palette | `Ctrl+Shift+P` |
| New workspace | `Ctrl+N` |
| Go to workspace (switcher) | `Ctrl+P` |
| Toggle sidebar | `Ctrl+B` |
| New surface (tab) | `Ctrl+T` |
| Close surface | `Ctrl+W` |
| Split right | `Ctrl+Shift+D` |
| Split down | `Ctrl+Shift+E` |
| Navigate panes | `Ctrl+Shift+Arrow` |
| Next workspace | `Ctrl+Shift+]` |
| Prev workspace | `Ctrl+Shift+[` |
| Zoom pane | `Ctrl+Shift+Z` |
| Find in terminal | `Ctrl+Shift+F` |
| Show notifications | `Ctrl+Shift+I` |
| Reopen previous session | `Ctrl+Shift+O` |
| Settings | `Ctrl+,` |

Two-step tmux-style chords are also supported (configurable in `~/.wmux/wmux.json`):

```json
{
  "shortcuts": {
    "bindings": {
      "newSurface": ["ctrl+b", "c"],
      "splitRight": ["ctrl+b", "%"],
      "splitDown": ["ctrl+b", "\""]
    }
  }
}
```

## Configuration

Files live under `~/.wmux/`:

- `wmux.json` — settings, keybindings, shell override
- `themes/*.json` — custom themes
- `session.json` — session snapshot for restore
- SQLite database — scrollback history

**Default shell:** auto-detected (PowerShell 7 > PowerShell 5 > cmd); override in Settings.

## Development

### Project Structure

```
wmux/
├── src/               # React frontend (components, stores, themes, lib)
├── src-tauri/         # Rust backend (PTY manager, SSH, persistence, IPC, notifications)
├── cli/               # Standalone wmux.exe CLI (named pipe client)
├── dist/              # Built frontend output
└── package.json
```

### Architecture

```
React frontend (xterm.js, Zustand, pane split tree)
        │  Tauri commands + channels
Rust backend (ConPTY via portable-pty, SSH via ssh2,
              OSC parser, SQLite persistence, named pipe server)
        │  \\.\pipe\wmux
wmux.exe CLI
```

### Testing

```powershell
npm run test                    # Frontend: components, stores, keybindings
cd src-tauri; cargo test        # Backend: PTY, persistence, OSC parsing, named pipe
```

### Useful Scripts

| Script | Description |
|---|---|
| `npm run dev` | Vite dev server (frontend only) |
| `npm run build` | Type-check + build frontend to `dist/` |
| `npm run tauri dev` | Full app in dev mode |
| `npm run tauri build` | Production build |
| `npm run test` | Run frontend test suite (Vitest) |

## Roadmap

| Phase | Description |
|---|---|
| 1 | Project scaffold (Tauri + React + TypeScript) |
| 2 | PTY manager + xterm.js terminal rendering |
| 3 | Pane splitting + workspace management + sidebar |
| 4 | Notification system (OSC parsing + rings + toasts) |
| 5 | Session persistence + agent resume hooks |
| 6 | CLI + named pipe API |
| 7 | Keybinding system + command palette |
| 8 | In-app browser pane (WebView2) |
| 9 | Theme engine + Windows polish (Mica, tray, animations) |

Workspace groups (multiple windows with linked workspaces) are planned for v2.

See [implementation_plan.md](implementation_plan.md) for the full design document.
