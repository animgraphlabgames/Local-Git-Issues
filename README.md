<div align="center">

# `Local Git Issues 📝`

[![Platform - Windows](https://img.shields.io/badge/Platform-Windows-0078D6?logo=windows&logoColor=white)](#)
[![License - MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

An offline, lightweight desktop issue tracker with a GitHub-inspired workflow, Markdown editing/shortcuts, and local SQLite storage.

</div>

## Features

- **Offline & Local-First**: all data is saved directly to a local SQLite database.
- **GitHub-Inspired UI**: familiar issue management with labels, projects, open/closed filters, and search.
- **Image Pasting**: paste screenshots directly via `Ctrl + V` or upload from disk. Compressed to WebP and stored locally.
- **Automated Storage Cleanup**: background garbage collection runs every hour and on startup to purge orphaned attachment files from disk.
- **Markdown Editor**: typical markdown workflow, syntax toolbar, and standard editing shortcuts.
- **Autostart**: run at Windows startup and minimize directly to system tray.
- **Keyboard-First Workflow**: fast navigation, quick-close shortcuts, and fuzzy search access.
## Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Ctrl + N` | Create new issue |
| `Ctrl + K` | Close / Reopen current issue |
| `Ctrl + L` | Focus issue search bar |
| `Ctrl + ,` | Open Settings |
| `Tab` | Toggle Open / Closed issues tab |
| `↑ / ↓` | Navigate issues in list |
| `Enter` | Open highlighted issue |
| `Ctrl + /` | Toggle keyboard shortcuts help |
| `Esc` | Close / Back to list |

## Development

### Prerequisites

Install Node.js, Rust, and pnpm via `winget`:

```powershell
winget install OpenJS.NodeJS.LTS Rustlang.Rustup pnpm.pnpm
```

### Run in Development
```bash
pnpm install
pnpm dev
```

### Build for Windows
```bash
pnpm build:win
```

The portable `.exe` file will be created at: `src-tauri/target/release/local-git-issues.exe`

## License

MIT