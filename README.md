# volt ![license] ![status]

[status]: https://img.shields.io/github/actions/workflow/status/hqnna/volt/release.yml?label=build&labelColor=4a414e
[license]: https://img.shields.io/github/license/hqnna/volt?labelColor=4a414e&color=3373cc

A TUI settings editor for the [Amp](https://ampcode.com) coding agent.

## Features

- **Grouped settings** — Browse settings organized into sections: General, Permissions, Tools, MCPs, and Advanced
- **Type-aware editing** — Booleans toggle inline, enums cycle through options, strings and numbers use a popup editor
- **`$EDITOR` integration** — Complex objects open in your `$EDITOR`; press `e` to force any setting into `$EDITOR`
- **Advanced section** — Add arbitrary key/value pairs for hidden or experimental settings
- **Validation** — Type-checks values before saving, with enum validation for constrained fields
- **Reset to default** — Press `r` to reset any setting; the key is removed from `settings.json` so the default applies
- **Preserves unknown keys** — Settings Volt doesn't know about are kept in the file and shown in Advanced

## Installation

### Pre-built binaries

Download a static binary from the [releases page](https://github.com/hqnna/volt/releases).

### Nix

```sh
# Run directly
nix run github:hqnna/volt

# Install to profile
nix profile install github:hqnna/volt

# Build a static binary (Linux only)
nix build github:hqnna/volt#volt-static
```

### From source

Requires a Rust toolchain (1.70+):

```sh
cargo install --git https://github.com/hqnna/volt
```

## Usage

```sh
# Edit the default settings file (~/.config/amp/settings.json)
volt

# Edit a specific file
volt --config /path/to/settings.json
```

## Keybindings

### Sidebar

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` / `Tab` | Switch to settings panel |
| `Ctrl+S` | Save |
| `q` | Quit |

### Settings Panel

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` | Toggle boolean / edit value / open item |
| `e` | Open current value in `$EDITOR` |
| `a` | Add item (arrays) or add custom key (Advanced) |
| `d` | Delete item (arrays) |
| `r` | Reset to default / remove custom key |
| `Tab` | Switch to sidebar |

### Popups

| Key | Action |
|-----|--------|
| `Enter` | Confirm |
| `Esc` | Cancel |
