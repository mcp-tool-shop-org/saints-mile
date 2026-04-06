---
title: Troubleshooting
description: Common issues and solutions for building and running Saint's Mile
sidebar:
  order: 9
---

## Build Issues

### Rust version too old

Saint's Mile requires **Rust 1.80 or later**. If you see errors about unsupported syntax or missing features:

```bash
rustup update stable
rustc --version  # Should show 1.80.0 or later
```

If you installed Rust through your system package manager, it may be outdated. Install via [rustup](https://rustup.rs/) instead.

### Missing toolchain

If `cargo build` fails with toolchain errors:

```bash
rustup toolchain install stable
rustup default stable
```

### Linker errors on Linux

Some Linux distributions need a C linker and development headers:

```bash
# Debian/Ubuntu
sudo apt install build-essential

# Fedora
sudo dnf groupinstall "Development Tools"

# Arch
sudo pacman -S base-devel
```

## Terminal Requirements

### Minimum terminal size

Saint's Mile needs a terminal window of at least **80 columns by 24 rows**. If the display looks broken or elements overlap, resize your terminal window or reduce the font size.

### Color support

The game requires **256-color support**. Most modern terminals support this out of the box. If colors look wrong:

- Make sure your `TERM` environment variable is set to something that supports 256 colors (e.g., `xterm-256color`)
- On older terminals, try `export TERM=xterm-256color` before launching the game

### UTF-8 encoding

Saint's Mile uses Unicode characters for UI elements. Your terminal must support UTF-8:

- **Linux/macOS:** Usually enabled by default. Check with `locale` — you should see `UTF-8` in the output.
- **Windows:** Use Windows Terminal (ships with Windows 11) or set your console code page with `chcp 65001`.

## Save File Location

Save files are stored in RON format (Rusty Object Notation) and are human-readable. Locations by platform:

| Platform | Path |
|----------|------|
| **Linux** | `~/.local/share/saints-mile/` |
| **macOS** | `~/Library/Application Support/saints-mile/` |
| **Windows** | `%APPDATA%\saints-mile\` |

You can back up, inspect, or delete save files directly. They are plain text.

## Known Platform Issues

### Windows

- **Legacy Command Prompt (`cmd.exe`):** May not render correctly. Use **Windows Terminal** instead, which ships with Windows 11 and is available from the Microsoft Store for Windows 10.
- **PowerShell ISE:** Not a real terminal emulator. Use Windows Terminal or standard PowerShell.
- **MSIX install:** If the MSIX package fails to install, make sure sideloading is enabled in Settings > Apps > Advanced app settings.

### macOS

- **Terminal.app:** Works but has limited color fidelity compared to alternatives. For the best experience, use [iTerm2](https://iterm2.com/) or [Alacritty](https://alacritty.org/).
- **macOS Gatekeeper:** If you downloaded a pre-built binary, macOS may block it. Right-click the binary and choose "Open" to bypass the warning, or run `xattr -cr saints-mile` to clear the quarantine flag.

### Linux

- **SSH sessions:** Work fine as long as the remote terminal supports 256 colors and UTF-8.
- **tmux/screen:** Fully supported. If colors look off, add `set -g default-terminal "xterm-256color"` to your tmux config.

## Reporting Bugs

If you run into an issue not covered here:

1. Check the [existing issues](https://github.com/mcp-tool-shop-org/saints-mile/issues) to see if it has already been reported
2. If not, [open a new bug report](https://github.com/mcp-tool-shop-org/saints-mile/issues/new?template=bug_report.md) with your platform info, steps to reproduce, and any save file or terminal output that might help
