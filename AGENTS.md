# Volt — TUI Settings Editor for Amp

## Overview

Volt is a terminal user interface (TUI) application for editing [Amp](https://ampcode.com)'s `settings.json` configuration file. Instead of hand-editing JSON, users get a structured, interactive editor with descriptions, type-aware inputs, and validation.

## Tech Stack

- **Language**: Rust (edition 2021)
- **TUI framework**: [ratatui](https://github.com/ratatui/ratatui) with [crossterm](https://github.com/crossterm-rs/crossterm) backend
- **CLI parsing**: [clap](https://github.com/clap-rs/clap) (derive)
- **Config**: [serde](https://serde.rs) + [serde_json](https://github.com/serde-rs/json)
- **Error handling**: [anyhow](https://github.com/dtolphin/anyhow)
- **Temp files**: [tempfile](https://github.com/Stebalien/tempfile) (for `$EDITOR` integration)
- **Platform paths**: [dirs](https://github.com/dirs-dev/dirs-rs)

## Architecture

```
src/
├── main.rs      — CLI parsing (clap), terminal setup/teardown, event loop, input handling
├── app.rs       — Application state (App), enums (Focus, InputMode, CustomKeyType), all mutation logic
├── config.rs    — Config struct: loading/saving settings.json, default path resolution per OS
├── editor.rs    — $EDITOR integration: write value to tempfile, spawn editor, read back
├── settings.rs  — Known settings registry: Section enum, SettingDef, SettingType, defaults, enum options
└── ui.rs        — Rendering: sidebar, settings panel, popups, status bar (all ratatui widgets)
```

## Development Environment

- **Nix flake** with [flake-parts](https://github.com/hercules-ci/flake-parts), [fenix](https://github.com/nix-community/fenix) (Rust toolchain), and [crane](https://github.com/ipetkov/crane) (Cargo builds in Nix)
- **direnv** with `use flake` for automatic shell activation
- All commands run through `nix develop`; building is done via `nix build`
- Static musl binaries for Linux via `nix build .#volt-static`

## Development Workflow

### Before every commit

1. `cargo fmt` — format code
2. `cargo check` — compilation check
3. `cargo clippy -- -D warnings` — lint (must pass with zero warnings)
4. `cargo test` — run all tests

All four must pass before committing.

### Commits

- Use [Conventional Commits](https://www.conventionalcommits.org/) format (e.g. `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`)
- Every change must be committed after it is verified
- Keep commits atomic — one logical change per commit

### Testing

- All functionality must have corresponding tests
- Tests live in `tests/` (integration) or inline `#[cfg(test)]` modules (unit)
- Tests must be meaningful — validate behavior, not just assert `true`
- Test both happy path and error/edge cases

## Settings File

- Default path: `~/.config/amp/settings.json` (Linux/macOS) or `%USERPROFILE%\.config\amp\settings.json` (Windows)
- Override with `--config <path>` CLI flag
- Unknown keys in the file are preserved and shown in the Advanced section

## UI Design

Two-panel layout: fixed-width sidebar (left) with section tabs, scrollable settings editor (right).

### Sections

| Section         | Contents                                                                                       |
|-----------------|------------------------------------------------------------------------------------------------|
| **General**     | Core settings: thinking, costs, notifications, git trailers, theme, update mode, etc.          |
| **Permissions** | `amp.permissions` — raw JSON editing via `$EDITOR`                                             |
| **Tools**       | `amp.tools.disable`, `amp.tools.stopTimeout`                                                   |
| **MCPs**        | `amp.mcpServers`, `amp.mcpPermissions` — raw JSON editing via `$EDITOR`                        |
| **Advanced**    | User-defined custom keys for hidden/experimental settings; also lists unknown keys from file    |

### Input Widgets by Type

| Type             | Widget                          | Editing method                                         |
|------------------|---------------------------------|--------------------------------------------------------|
| `boolean`        | Toggle / checkbox               | `Enter` to toggle inline                               |
| `string`         | Text input                      | Popup textbox                                          |
| `string` (enum)  | Dropdown / select               | Cycle through options inline                           |
| `number`         | Numeric input                   | Popup textbox                                          |
| `array<string>`  | List with add/delete            | `a` to add item, `d` to delete selected item           |
| `object`         | Label + "edit" action           | Opens `$EDITOR` with JSON                              |
| `array<object>`  | List with add/delete + edit     | `a`/`d` for items; editing opens `$EDITOR`             |

### Key Bindings

- `↑`/`k`, `↓`/`j` — navigate
- `Tab` — switch focus between sidebar and settings panel
- `Enter` — toggle boolean / edit value / switch to settings from sidebar
- `e` — force open any value in `$EDITOR`
- `a` — add item (arrays) or add custom key (Advanced)
- `d` — delete item (arrays)
- `r` — reset to default (removes key from settings.json)
- `Ctrl+S` — save
- `q` / `Ctrl+C` — quit

## Known Amp Settings

### Shared (Editor Extension and CLI)

| Key | Type | Default |
|-----|------|---------|
| `amp.anthropic.thinking.enabled` | `boolean` | `true` |
| `amp.fuzzy.alwaysIncludePaths` | `array<string>` | `[]` |
| `amp.permissions` | `array<object>` | `[]` |
| `amp.showCosts` | `boolean` | `true` |
| `amp.git.commit.ampThread.enabled` | `boolean` | `true` |
| `amp.git.commit.coauthor.enabled` | `boolean` | `true` |
| `amp.mcpServers` | `object` | `{}` |
| `amp.defaultVisibility` | `object` | `{}` |
| `amp.bitbucketToken` | `string` | `""` |
| `amp.notifications.enabled` | `boolean` | `true` |
| `amp.skills.path` | `string` | `""` |
| `amp.terminal.commands.nodeSpawn.loadProfile` | `string` (enum: `always`, `never`, `daily`) | `"always"` |
| `amp.tools.disable` | `array<string>` | `[]` |
| `amp.tools.stopTimeout` | `number` | `300` |
| `amp.mcpPermissions` | `array<object>` | `[]` |
| `amp.tab.clipboard.enabled` | `boolean` | _(unknown)_ |
| `amp.terminal.theme` | `string` (enum: `terminal`, `dark`, `light`, `catppuccin-mocha`, `solarized-dark`, `solarized-light`, `gruvbox-dark-hard`, `nord`, or custom) | `"terminal"` |

### CLI-only

| Key | Type | Default |
|-----|------|---------|
| `amp.updates.mode` | `string` (enum: `auto`, `warn`, `disabled`) | `"auto"` |
| `amp.internal.deepReasoningEffort` | `string` (enum: `medium`, `high`, `xhigh`) | `"medium"` |

### Hidden / Experimental

- `amp.experimental.modes` — not shown in normal UI, accessible via Advanced section

## Non-Goals

- Managing MCP server configs in detail (just raw JSON editing for `amp.mcpServers`)
- Managing permissions rules in detail (just raw JSON editing for `amp.permissions`)
- Replacing Amp's own CLI config commands
- Editing workspace-level or enterprise managed settings
