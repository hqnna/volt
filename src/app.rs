//! Application state and logic for the Volt TUI.

use crate::config::Config;
use crate::settings::{self, Section, SettingType};
use serde_json::Value;

/// Which panel currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    Settings,
}

/// Application state.
pub struct App {
    pub config: Config,
    pub selected_section: usize,
    pub selected_setting: usize,
    pub focus: Focus,
    pub should_quit: bool,
    pub status_message: Option<String>,
    /// Whether the user is currently inline-editing a string/number field.
    pub editing: bool,
    /// Buffer for inline text editing.
    pub edit_buffer: String,
}

impl App {
    /// Creates a new App from a loaded config.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            selected_section: 0,
            selected_setting: 0,
            focus: Focus::Sidebar,
            should_quit: false,
            status_message: None,
            editing: false,
            edit_buffer: String::new(),
        }
    }

    /// Returns the currently selected section.
    pub fn current_section(&self) -> Section {
        Section::ALL[self.selected_section]
    }

    /// Returns the settings list for the current section.
    pub fn current_settings(&self) -> Vec<SettingEntry> {
        let section = self.current_section();
        match section {
            Section::Advanced => self.advanced_entries(),
            _ => settings::settings_for_section(section)
                .into_iter()
                .map(SettingEntry::Known)
                .collect(),
        }
    }

    /// Returns entries for the Advanced section (unknown keys).
    fn advanced_entries(&self) -> Vec<SettingEntry> {
        self.config
            .unknown_keys()
            .into_iter()
            .map(SettingEntry::Unknown)
            .collect()
    }

    /// Returns the number of items in the current section.
    pub fn current_item_count(&self) -> usize {
        self.current_settings().len()
    }

    /// Moves selection up in the current panel.
    pub fn move_up(&mut self) {
        match self.focus {
            Focus::Sidebar => {
                if self.selected_section > 0 {
                    self.selected_section -= 1;
                    self.selected_setting = 0;
                }
            }
            Focus::Settings => {
                if self.selected_setting > 0 {
                    self.selected_setting -= 1;
                }
            }
        }
    }

    /// Moves selection down in the current panel.
    pub fn move_down(&mut self) {
        match self.focus {
            Focus::Sidebar => {
                if self.selected_section < Section::ALL.len() - 1 {
                    self.selected_section += 1;
                    self.selected_setting = 0;
                }
            }
            Focus::Settings => {
                let count = self.current_item_count();
                if count > 0 && self.selected_setting < count - 1 {
                    self.selected_setting += 1;
                }
            }
        }
    }

    /// Toggles focus between sidebar and settings panel.
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Sidebar => Focus::Settings,
            Focus::Settings => Focus::Sidebar,
        };
    }

    /// Handles Enter key on the currently selected setting.
    pub fn activate_setting(&mut self) {
        let entries = self.current_settings();
        let Some(entry) = entries.get(self.selected_setting) else {
            return;
        };

        match entry {
            SettingEntry::Known(def) => match def.setting_type {
                SettingType::Boolean => {
                    let current = self.config.get(def.key);
                    let toggled = !current.as_bool().unwrap_or(false);
                    self.config.set(def.key, Value::Bool(toggled));
                }
                SettingType::String | SettingType::Number => {
                    self.editing = true;
                    let current = self.config.get(def.key);
                    self.edit_buffer = match &current {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        _ => String::new(),
                    };
                }
                SettingType::StringEnum => {
                    self.cycle_enum(def);
                }
                _ => {}
            },
            SettingEntry::Unknown(_) => {}
        }
    }

    /// Cycles through enum options for a StringEnum setting.
    fn cycle_enum(&mut self, def: &settings::SettingDef) {
        let Some(options) = def.enum_options else {
            return;
        };
        let current = self.config.get(def.key);
        let current_str = current.as_str().unwrap_or("");
        let current_idx = options.iter().position(|o| *o == current_str);
        let next_idx = match current_idx {
            Some(i) => (i + 1) % options.len(),
            None => 0,
        };
        self.config
            .set(def.key, Value::String(options[next_idx].to_string()));
    }

    /// Commits the current inline edit.
    pub fn commit_edit(&mut self) {
        if !self.editing {
            return;
        }
        self.editing = false;

        let entries = self.current_settings();
        let Some(entry) = entries.get(self.selected_setting) else {
            return;
        };

        let SettingEntry::Known(def) = entry else {
            return;
        };

        let value = match def.setting_type {
            SettingType::Number => {
                if let Ok(n) = self.edit_buffer.parse::<i64>() {
                    Value::Number(n.into())
                } else if let Ok(n) = self.edit_buffer.parse::<f64>() {
                    if let Some(n) = serde_json::Number::from_f64(n) {
                        Value::Number(n)
                    } else {
                        self.status_message = Some("Invalid number".to_string());
                        return;
                    }
                } else {
                    self.status_message = Some("Invalid number".to_string());
                    return;
                }
            }
            _ => Value::String(self.edit_buffer.clone()),
        };

        if let Err(e) = Config::validate_value(def.key, &value) {
            self.status_message = Some(e.to_string());
            return;
        }

        self.config.set(def.key, value);
        self.edit_buffer.clear();
    }

    /// Cancels the current inline edit.
    pub fn cancel_edit(&mut self) {
        self.editing = false;
        self.edit_buffer.clear();
    }

    /// Resets the currently selected setting to its default.
    pub fn reset_setting(&mut self) {
        let entries = self.current_settings();
        let Some(entry) = entries.get(self.selected_setting) else {
            return;
        };

        match entry {
            SettingEntry::Known(def) => {
                self.config.remove(def.key);
                self.status_message = Some(format!("Reset {} to default", def.key));
            }
            SettingEntry::Unknown(key) => {
                self.config.remove(key);
                self.status_message = Some(format!("Removed {}", key));
                // Adjust selection if needed
                let count = self.current_item_count();
                if count > 0 && self.selected_setting >= count {
                    self.selected_setting = count - 1;
                }
            }
        }
    }

    /// Saves the configuration to disk.
    pub fn save(&mut self) {
        match self.config.save() {
            Ok(()) => self.status_message = Some("Saved!".to_string()),
            Err(e) => self.status_message = Some(format!("Save failed: {e}")),
        }
    }
}

/// An entry in the settings list — either a known setting or an unknown key.
#[derive(Debug, Clone)]
pub enum SettingEntry {
    Known(settings::SettingDef),
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn test_app() -> App {
        let mut f = NamedTempFile::new().unwrap();
        write!(
            f,
            r#"{{
    "amp.showCosts": true,
    "amp.notifications.enabled": false,
    "amp.experimental.modes": ["bombadil"]
}}"#
        )
        .unwrap();
        let config = Config::load(f.path()).unwrap();
        App::new(config)
    }

    #[test]
    fn test_initial_state() {
        let app = test_app();
        assert_eq!(app.current_section(), Section::General);
        assert_eq!(app.selected_setting, 0);
        assert_eq!(app.focus, Focus::Sidebar);
        assert!(!app.should_quit);
        assert!(!app.editing);
    }

    #[test]
    fn test_navigate_sections() {
        let mut app = test_app();
        assert_eq!(app.current_section(), Section::General);

        app.move_down();
        assert_eq!(app.current_section(), Section::Permissions);

        app.move_down();
        assert_eq!(app.current_section(), Section::Tools);

        app.move_up();
        assert_eq!(app.current_section(), Section::Permissions);
    }

    #[test]
    fn test_toggle_focus() {
        let mut app = test_app();
        assert_eq!(app.focus, Focus::Sidebar);
        app.toggle_focus();
        assert_eq!(app.focus, Focus::Settings);
        app.toggle_focus();
        assert_eq!(app.focus, Focus::Sidebar);
    }

    #[test]
    fn test_toggle_boolean() {
        let mut app = test_app();
        app.focus = Focus::Settings;
        // First setting in General is amp.anthropic.thinking.enabled (default true)
        app.activate_setting();
        assert_eq!(
            app.config.get("amp.anthropic.thinking.enabled"),
            Value::Bool(false)
        );
        app.activate_setting();
        assert_eq!(
            app.config.get("amp.anthropic.thinking.enabled"),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_cycle_enum() {
        let mut app = test_app();
        app.focus = Focus::Settings;
        // Navigate to amp.terminal.theme (a StringEnum)
        let entries = app.current_settings();
        let theme_idx = entries
            .iter()
            .position(|e| matches!(e, SettingEntry::Known(d) if d.key == "amp.terminal.theme"))
            .unwrap();
        app.selected_setting = theme_idx;

        // Default is empty string, cycling should go to first option
        app.activate_setting();
        assert_eq!(
            app.config.get("amp.terminal.theme"),
            Value::String("terminal".to_string())
        );

        app.activate_setting();
        assert_eq!(
            app.config.get("amp.terminal.theme"),
            Value::String("dark".to_string())
        );
    }

    #[test]
    fn test_reset_setting() {
        let mut app = test_app();
        app.focus = Focus::Settings;

        // notifications.enabled is set to false in our test data
        let entries = app.current_settings();
        let idx = entries
            .iter()
            .position(
                |e| matches!(e, SettingEntry::Known(d) if d.key == "amp.notifications.enabled"),
            )
            .unwrap();
        app.selected_setting = idx;

        assert_eq!(
            app.config.get("amp.notifications.enabled"),
            Value::Bool(false)
        );

        app.reset_setting();
        // Should fall back to default (true)
        assert_eq!(
            app.config.get("amp.notifications.enabled"),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_advanced_shows_unknown_keys() {
        let mut app = test_app();
        // Navigate to Advanced section
        app.selected_section = 4; // Advanced is index 4
        assert_eq!(app.current_section(), Section::Advanced);

        let entries = app.current_settings();
        assert!(entries
            .iter()
            .any(|e| matches!(e, SettingEntry::Unknown(k) if k == "amp.experimental.modes")));
    }

    #[test]
    fn test_move_bounds() {
        let mut app = test_app();
        // At top, moving up should stay
        app.move_up();
        assert_eq!(app.selected_section, 0);

        // Move to bottom
        for _ in 0..10 {
            app.move_down();
        }
        assert_eq!(app.selected_section, Section::ALL.len() - 1);

        // Further down should stay
        app.move_down();
        assert_eq!(app.selected_section, Section::ALL.len() - 1);
    }

    #[test]
    fn test_section_change_resets_setting_index() {
        let mut app = test_app();
        app.focus = Focus::Settings;
        app.selected_setting = 5;
        app.focus = Focus::Sidebar;
        app.move_down();
        assert_eq!(app.selected_setting, 0);
    }

    #[test]
    fn test_inline_edit_string() {
        let mut app = test_app();
        app.focus = Focus::Settings;
        // Navigate to amp.bitbucketToken (a string)
        let entries = app.current_settings();
        let idx = entries
            .iter()
            .position(|e| matches!(e, SettingEntry::Known(d) if d.key == "amp.bitbucketToken"))
            .unwrap();
        app.selected_setting = idx;

        app.activate_setting();
        assert!(app.editing);
        app.edit_buffer = "my-token".to_string();
        app.commit_edit();
        assert!(!app.editing);
        assert_eq!(
            app.config.get("amp.bitbucketToken"),
            Value::String("my-token".to_string())
        );
    }

    #[test]
    fn test_inline_edit_number() {
        let mut app = test_app();
        app.focus = Focus::Settings;
        // Navigate to Tools section
        app.selected_section = 2; // Tools
        let entries = app.current_settings();
        let idx = entries
            .iter()
            .position(|e| matches!(e, SettingEntry::Known(d) if d.key == "amp.tools.stopTimeout"))
            .unwrap();
        app.selected_setting = idx;

        app.activate_setting();
        assert!(app.editing);
        app.edit_buffer = "120".to_string();
        app.commit_edit();
        assert!(!app.editing);
        assert_eq!(
            app.config.get("amp.tools.stopTimeout"),
            Value::Number(120.into())
        );
    }

    #[test]
    fn test_inline_edit_cancel() {
        let mut app = test_app();
        app.editing = true;
        app.edit_buffer = "something".to_string();
        app.cancel_edit();
        assert!(!app.editing);
        assert!(app.edit_buffer.is_empty());
    }
}
