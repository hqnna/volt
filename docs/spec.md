# Volt — TUI Settings Editor for Amp

## Overview

Volt is a terminal user interface (TUI) application for editing [Amp](https://ampcode.com)'s `settings.json` configuration file. Instead of hand-editing JSON, users get a structured, interactive editor with descriptions, type-aware inputs, and validation.

## Goals

- Provide a friendly TUI to browse and edit all **documented** Amp settings
- Read/write `~/.config/amp/settings.json` (Linux/macOS) or `%USERPROFILE%\.config\amp\settings.json` (Windows)
- Support an **Advanced / Custom Keys** section for hidden/experimental settings (arbitrary key + type + value)
- Validate values by type before writing

## Non-Goals

- Managing MCP server configs in detail (complex nested objects — just raw JSON editing for `amp.mcpServers`)
- Managing permissions rules in detail (complex array of objects — just raw JSON editing for `amp.permissions`)
- Replacing Amp's own CLI config commands
- Editing workspace-level or enterprise managed settings

## Target Users

- Amp CLI users who want a quick way to tweak settings without hand-editing JSON

---

## Known Settings (from ampcode.com/manual)

### Editor Extension and CLI (shared)

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `amp.anthropic.thinking.enabled` | `boolean` | `true` | Enable Claude's extended thinking capabilities |
| `amp.fuzzy.alwaysIncludePaths` | `array<string>` | `[]` | Glob patterns always included in fuzzy file search (even if gitignored) |
| `amp.permissions` | `array<object>` | `[]` | Tool permission rules (allow/reject/ask). Complex object — raw JSON edit. |
| `amp.showCosts` | `boolean` | `true` | Show cost information for threads |
| `amp.git.commit.ampThread.enabled` | `boolean` | `true` | Add `Amp-Thread:` trailer to git commits |
| `amp.git.commit.coauthor.enabled` | `boolean` | `true` | Add `Co-authored-by: Amp` trailer to git commits |
| `amp.mcpServers` | `object` | `{}` | MCP server definitions. Complex object — raw JSON edit. |
| `amp.defaultVisibility` | `object` | `{}` | Default thread visibility per repo origin. Values: `private`, `public`, `workspace`, `group`. |
| `amp.bitbucketToken` | `string` | `""` | Personal access token for Bitbucket Enterprise |
| `amp.notifications.enabled` | `boolean` | `true` | Play notification sounds on task completion / input needed |
| `amp.skills.path` | `string` | `""` | Colon-separated paths to additional skill directories |
| `amp.terminal.commands.nodeSpawn.loadProfile` | `string` (enum) | `"always"` | Load env from shell profile. Options: `"always"`, `"never"`, `"daily"` |
| `amp.tools.disable` | `array<string>` | `[]` | Disable specific tools by name (glob patterns supported) |
| `amp.tools.stopTimeout` | `number` | `300` | Seconds before canceling a running tool |
| `amp.mcpPermissions` | `array<object>` | `[]` | Allow/block MCP servers by pattern. Complex object — raw JSON edit. |
| `amp.tab.clipboard.enabled` | `boolean` | _(unknown)_ | Clipboard integration for tab (seen in user config, undocumented publicly) |
| `amp.terminal.theme` | `string` | `"terminal"` | CLI theme name |

### CLI-only

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `amp.updates.mode` | `string` (enum) | `"auto"` | Update behavior. Options: `"auto"`, `"warn"`, `"disabled"` |
| `amp.internal.deepReasoningEffort` | `string` (enum) | `"medium"` | Deep mode reasoning effort. Options: `"medium"`, `"high"`, `"xhigh"` |

### Hidden / Experimental (NOT shown in normal UI)

These exist but are intentionally excluded from the main settings list:

- `amp.experimental.modes`

Users can still set these via the **Advanced** section (see below).

---

## UI Design

### Layout

Two-panel layout: a fixed-width sidebar on the left with section tabs, and the settings editor on the right.

```
┌ Volt ──────────┬────────────────────────────────────┐
│                │                                    │
│  ┌────────────┐│                                    │
│  │ General    ││   amp.showCosts            [✓]     │
│  └────────────┘│                                    │
│  ┌────────────┐│   amp.notifications        [✗]     │
│  │ Permissions││                                    │
│  └────────────┘│   amp.tools.stopTimeout  [ 300 ]   │
│  ┌────────────┐│                                    │
│  │ Tools      ││   ...                              │
│  └────────────┘│                                    │
│  ┌────────────┐│                                    │
│  │ MCPs       ││                                    │
│  └────────────┘│                                    │
│  ┌────────────┐│                                    │
│  │ Advanced   ││                                    │
│  └────────────┘│                                    │
│                │                                    │
│                │                                    │
└────────────────┴────────────────────────────────────┘
```

- **Left panel**: Fixed-width sidebar with bordered tab buttons for each section. "Volt" appears as the border title of the left panel.
- **Right panel**: Scrollable settings list for the currently selected section. For single-key sections (e.g. Permissions), the right panel is just a full editor for that key instead of a settings list.
- Navigation: `↑`/`↓` to move between items, `Tab` to switch focus between sidebar and settings panel, `Enter` to toggle/edit a setting.
- The selected tab is visually highlighted.

### Sections & Grouping

| Section | Keys |
|---------|------|
| **General** | `amp.anthropic.thinking.enabled`, `amp.anthropic.effort`, `amp.showCosts`, `amp.notifications.enabled`, `amp.defaultVisibility`, `amp.bitbucketToken`, `amp.fuzzy.alwaysIncludePaths`, `amp.skills.path`, `amp.git.commit.ampThread.enabled`, `amp.git.commit.coauthor.enabled`, `amp.terminal.theme`, `amp.terminal.commands.nodeSpawn.loadProfile`, `amp.updates.mode`, `amp.internal.deepReasoningEffort`, `amp.tab.clipboard.enabled` |
| **Permissions** | `amp.permissions` |
| **Tools** | `amp.tools.disable`, `amp.tools.stopTimeout` |
| **MCPs** | `amp.mcpServers`, `amp.mcpPermissions` |
| **Advanced** | User-defined custom keys (for hidden/experimental settings) |

### Input Widgets by Type

| Type | Widget | Editing method |
|------|--------|----------------|
| `boolean` | Toggle / checkbox | Inline — `Enter` to toggle |
| `string` | Text input | Inline — type directly |
| `string` (enum) | Dropdown / select list | Inline — cycle or pick from list |
| `number` | Numeric input | Inline — type directly |
| `array<string>` | List with add/delete | Inline — `a` to add item, `d` to delete selected item |
| `object` | Label + "edit" action | Opens `$EDITOR` with JSON, waits for save & quit, then updates value |
| `array<object>` | List with add/delete + edit | Inline add/delete like string arrays; editing an individual item opens `$EDITOR` |

### Editing Interactions

- **Inline editing**: Booleans, strings, numbers, enums, and string arrays are all edited directly in the TUI without leaving it.
- **`$EDITOR` for JSON**: Plain `object` values open the whole object in `$EDITOR`. For `array<object>`, individual items open in `$EDITOR` when selected for editing. Volt suspends, waits for the editor to exit, reads back the temp file, validates it, and updates the value.
- **Force `$EDITOR`**: A keybinding (e.g. `e`) opens *any* setting's current value in `$EDITOR`, even simple types. Useful for power users or bulk-editing arrays.
- **Popup text input**: When a text value is needed (e.g. entering a new key name in Advanced, or adding an item to a string array), a centered modal textbox appears over the TUI.

### Advanced Section

For hidden/experimental keys (or any unknown key):

1. User presses "add" — a **centered popup textbox** prompts for the key name
2. User selects a value type: `boolean`, `string`, `number`, `array`, `object`
3. Appropriate inline widget or `$EDITOR` appears based on chosen type
4. Value is saved into `settings.json` alongside known keys

Existing unknown keys found in the loaded `settings.json` are automatically listed here.

---

## Behavior

- **Load**: Read `settings.json` on startup. Missing keys use defaults. Unknown keys go to Advanced.
- **Edit**: Changes are held in memory until explicitly saved.
- **Save**: Write the full settings back to `settings.json` as formatted JSON.
- **Validation**: Type-check values before save. Show inline errors.
- **Preserve unknown keys**: Never delete keys from the file that Volt doesn't know about.

---

## Tech Stack

- **Language**: Rust
- **TUI framework**: [ratatui](https://github.com/ratatui/ratatui)

## Development Environment

- **Nix flake** with [flake-parts](https://github.com/hercules-ci/flake-parts), [fenix](https://github.com/nix-community/fenix) (Rust toolchain), and [crane](https://github.com/ipetkov/crane) (Cargo builds in Nix)
- **direnv** with `use flake` for automatic shell activation
- All commands run through `nix develop`; building is done via `nix build`

## Development Workflow

### Before every commit

1. `cargo fmt` — format code
2. `cargo check` — compilation check
3. `cargo clippy -- -D warnings` — lint
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

---

## Resolved Decisions

- **`--config <path>` flag**: Supported. Optional CLI flag to override the settings file location.
- **OS auto-detection**: Volt auto-resolves the default settings path based on OS:
  - Linux: `~/.config/amp/settings.json`
  - macOS: `~/.config/amp/settings.json`
  - Windows: `%USERPROFILE%\.config\amp\settings.json`
- **Reset to default**: Each setting has a "reset to default" action (e.g. keybinding `r`) that restores the documented default value. Resets remove the key from `settings.json` (absence = default).
- **Theme picker**: `amp.terminal.theme` uses a dropdown with the following built-in options:
  - `terminal` (default)
  - `dark`
  - `light`
  - `catppuccin-mocha`
  - `solarized-dark`
  - `solarized-light`
  - `gruvbox-dark-hard`
  - `nord`
  - `Custom` — selecting this prompts a popup textbox for the user to type a custom theme name
- **`amp.tab.clipboard.enabled`**: Kept in General (not Advanced) since it appears in user configs even if undocumented.
