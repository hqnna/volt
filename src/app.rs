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

/// A request to open an external editor, returned from app methods.
#[derive(Debug, Clone)]
pub struct EditorRequest {
    /// The setting key being edited.
    pub key: String,
    /// The current value to edit.
    pub value: Value,
    /// For array<object>, the index of the item being edited.
    pub array_index: Option<usize>,
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
        if self.current_section().is_single_key() {
            self.single_key_item_count()
        } else {
            self.current_settings().len()
        }
    }

    /// Returns the number of array items for a single-key section.
    fn single_key_item_count(&self) -> usize {
        let entries = self.current_settings();
        match entries.first() {
            Some(SettingEntry::Known(def)) => {
                self.config.get(def.key).as_array().map_or(0, |a| a.len())
            }
            _ => 0,
        }
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
    /// Returns an `EditorRequest` if the setting needs to be opened in `$EDITOR`.
    pub fn activate_setting(&mut self) -> Option<EditorRequest> {
        if self.current_section().is_single_key() {
            return self.activate_single_key_item();
        }

        let entries = self.current_settings();
        let entry = entries.get(self.selected_setting)?;

        match entry {
            SettingEntry::Known(def) => match def.setting_type {
                SettingType::Boolean => {
                    let current = self.config.get(def.key);
                    let toggled = !current.as_bool().unwrap_or(false);
                    self.config.set(def.key, Value::Bool(toggled));
                    None
                }
                SettingType::String | SettingType::Number => {
                    self.editing = true;
                    let current = self.config.get(def.key);
                    self.edit_buffer = match &current {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        _ => String::new(),
                    };
                    None
                }
                SettingType::StringEnum => {
                    self.cycle_enum(def);
                    None
                }
                SettingType::Object => Some(EditorRequest {
                    key: def.key.to_string(),
                    value: self.config.get(def.key),
                    array_index: None,
                }),
                SettingType::ArrayObject => {
                    let arr = self.config.get(def.key);
                    let items = arr.as_array().cloned().unwrap_or_default();
                    if items.is_empty() {
                        self.status_message =
                            Some("Empty array. Press 'a' to add an item.".to_string());
                        None
                    } else {
                        let idx = 0;
                        Some(EditorRequest {
                            key: def.key.to_string(),
                            value: items[idx].clone(),
                            array_index: Some(idx),
                        })
                    }
                }
                SettingType::ArrayString => {
                    self.status_message =
                        Some("Press 'a' to add, 'd' to delete items.".to_string());
                    None
                }
            },
            SettingEntry::Unknown(key) => {
                let value = self.config.get(key);
                Some(EditorRequest {
                    key: key.clone(),
                    value,
                    array_index: None,
                })
            }
        }
    }

    /// Activates the selected array item in a single-key section.
    fn activate_single_key_item(&self) -> Option<EditorRequest> {
        let entries = self.current_settings();
        let def = match entries.first() {
            Some(SettingEntry::Known(def)) => def,
            _ => return None,
        };
        let arr = self.config.get(def.key);
        let items = arr.as_array().cloned().unwrap_or_default();
        let item = items.get(self.selected_setting)?;
        Some(EditorRequest {
            key: def.key.to_string(),
            value: item.clone(),
            array_index: Some(self.selected_setting),
        })
    }

    /// Forces opening the current setting in `$EDITOR`.
    pub fn force_editor(&self) -> Option<EditorRequest> {
        let entries = self.current_settings();
        let entry = if self.current_section().is_single_key() {
            entries.first()?
        } else {
            entries.get(self.selected_setting)?
        };

        let (key, value) = match entry {
            SettingEntry::Known(def) => (def.key.to_string(), self.config.get(def.key)),
            SettingEntry::Unknown(key) => (key.clone(), self.config.get(key)),
        };

        Some(EditorRequest {
            key,
            value,
            array_index: None,
        })
    }

    /// Applies the result from an external editor back to the config.
    pub fn apply_editor_result(&mut self, request: &EditorRequest, edited: Value) {
        match request.array_index {
            Some(idx) => {
                let mut arr = self
                    .config
                    .get(&request.key)
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                if idx < arr.len() {
                    arr[idx] = edited;
                }
                self.config.set(&request.key, Value::Array(arr));
            }
            None => {
                self.config.set(&request.key, edited);
            }
        }
        self.status_message = Some(format!("Updated {}", request.key));
    }

    /// Adds an item to a string array setting (prompts for value via edit buffer).
    pub fn add_array_item(&mut self) {
        let def = self.selected_array_def();
        let Some(def) = def else {
            return;
        };

        match def.setting_type {
            SettingType::ArrayString | SettingType::ArrayObject => {
                self.editing = true;
                self.edit_buffer.clear();
            }
            _ => {}
        }
    }

    /// Deletes an item from an array setting.
    /// In single-key sections, deletes the selected item; otherwise deletes the last.
    pub fn delete_array_item(&mut self) {
        let section = self.current_section();
        let def = self.selected_array_def();
        let Some(def) = def else {
            return;
        };

        match def.setting_type {
            SettingType::ArrayString | SettingType::ArrayObject => {
                let mut arr = self
                    .config
                    .get(def.key)
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                if arr.is_empty() {
                    self.status_message = Some("Array is already empty.".to_string());
                } else if section.is_single_key() {
                    let idx = self.selected_setting.min(arr.len() - 1);
                    arr.remove(idx);
                    self.config.set(def.key, Value::Array(arr.clone()));
                    self.status_message = Some(format!("Removed item {} from {}", idx, def.key));
                    if !arr.is_empty() && self.selected_setting >= arr.len() {
                        self.selected_setting = arr.len() - 1;
                    }
                } else {
                    arr.pop();
                    self.config.set(def.key, Value::Array(arr));
                    self.status_message = Some(format!("Removed last item from {}", def.key));
                }
            }
            _ => {}
        }
    }

    /// Returns the SettingDef for the currently selected array setting.
    /// In single-key sections, returns the section's only setting.
    /// In multi-key sections, returns the selected setting if it's an array type.
    fn selected_array_def(&self) -> Option<settings::SettingDef> {
        let entries = self.current_settings();
        let entry = if self.current_section().is_single_key() {
            entries.first()
        } else {
            entries.get(self.selected_setting)
        };
        match entry {
            Some(SettingEntry::Known(def))
                if matches!(
                    def.setting_type,
                    SettingType::ArrayString | SettingType::ArrayObject
                ) =>
            {
                Some(def.clone())
            }
            _ => None,
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
        let entry = if self.current_section().is_single_key() {
            entries.first()
        } else {
            entries.get(self.selected_setting)
        };
        let Some(entry) = entry else {
            return;
        };

        let SettingEntry::Known(def) = entry else {
            return;
        };

        match def.setting_type {
            SettingType::ArrayString => {
                if !self.edit_buffer.is_empty() {
                    let mut arr = self
                        .config
                        .get(def.key)
                        .as_array()
                        .cloned()
                        .unwrap_or_default();
                    arr.push(Value::String(self.edit_buffer.clone()));
                    self.config.set(def.key, Value::Array(arr));
                    self.status_message = Some(format!("Added item to {}", def.key));
                }
                self.edit_buffer.clear();
                return;
            }
            SettingType::ArrayObject => {
                if !self.edit_buffer.is_empty() {
                    match serde_json::from_str::<Value>(&self.edit_buffer) {
                        Ok(val) if val.is_object() => {
                            let mut arr = self
                                .config
                                .get(def.key)
                                .as_array()
                                .cloned()
                                .unwrap_or_default();
                            arr.push(val);
                            self.config.set(def.key, Value::Array(arr));
                            self.status_message = Some(format!("Added item to {}", def.key));
                        }
                        Ok(_) => {
                            self.status_message = Some("Value must be a JSON object".to_string());
                        }
                        Err(e) => {
                            self.status_message = Some(format!("Invalid JSON: {e}"));
                        }
                    }
                }
                self.edit_buffer.clear();
                return;
            }
            _ => {}
        }

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
        let entry = if self.current_section().is_single_key() {
            entries.first()
        } else {
            entries.get(self.selected_setting)
        };
        let Some(entry) = entry else {
            return;
        };

        match entry {
            SettingEntry::Known(def) => {
                self.config.remove(def.key);
                self.status_message = Some(format!("Reset {} to default", def.key));
                if self.current_section().is_single_key() {
                    self.selected_setting = 0;
                }
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

    #[test]
    fn test_object_returns_editor_request() {
        let mut app = test_app();
        app.focus = Focus::Settings;
        let entries = app.current_settings();
        let idx = entries
            .iter()
            .position(|e| matches!(e, SettingEntry::Known(d) if d.key == "amp.defaultVisibility"))
            .unwrap();
        app.selected_setting = idx;

        let req = app.activate_setting();
        assert!(req.is_some());
        let req = req.unwrap();
        assert_eq!(req.key, "amp.defaultVisibility");
        assert!(req.array_index.is_none());
    }

    #[test]
    fn test_array_string_add_item() {
        let mut app = test_app();
        app.focus = Focus::Settings;
        let entries = app.current_settings();
        let idx = entries
            .iter()
            .position(
                |e| matches!(e, SettingEntry::Known(d) if d.key == "amp.fuzzy.alwaysIncludePaths"),
            )
            .unwrap();
        app.selected_setting = idx;

        app.add_array_item();
        assert!(app.editing);
        app.edit_buffer = "*.rs".to_string();
        app.commit_edit();
        assert!(!app.editing);
        assert_eq!(
            app.config.get("amp.fuzzy.alwaysIncludePaths"),
            Value::Array(vec![Value::String("*.rs".into())])
        );
    }

    #[test]
    fn test_array_string_delete_item() {
        let mut app = test_app();
        app.focus = Focus::Settings;
        app.config.set(
            "amp.fuzzy.alwaysIncludePaths",
            Value::Array(vec![Value::String("a".into()), Value::String("b".into())]),
        );
        let entries = app.current_settings();
        let idx = entries
            .iter()
            .position(
                |e| matches!(e, SettingEntry::Known(d) if d.key == "amp.fuzzy.alwaysIncludePaths"),
            )
            .unwrap();
        app.selected_setting = idx;

        app.delete_array_item();
        assert_eq!(
            app.config.get("amp.fuzzy.alwaysIncludePaths"),
            Value::Array(vec![Value::String("a".into())])
        );
    }

    #[test]
    fn test_delete_empty_array() {
        let mut app = test_app();
        app.focus = Focus::Settings;
        let entries = app.current_settings();
        let idx = entries
            .iter()
            .position(
                |e| matches!(e, SettingEntry::Known(d) if d.key == "amp.fuzzy.alwaysIncludePaths"),
            )
            .unwrap();
        app.selected_setting = idx;

        app.delete_array_item();
        assert!(app.status_message.is_some());
        assert!(app.status_message.unwrap().contains("empty"));
    }

    #[test]
    fn test_force_editor() {
        let mut app = test_app();
        app.focus = Focus::Settings;
        // Any setting should produce an EditorRequest
        let req = app.force_editor();
        assert!(req.is_some());
    }

    #[test]
    fn test_apply_editor_result() {
        let mut app = test_app();
        let req = EditorRequest {
            key: "amp.defaultVisibility".to_string(),
            value: Value::Object(serde_json::Map::new()),
            array_index: None,
        };
        let mut map = serde_json::Map::new();
        map.insert("origin".into(), Value::String("private".into()));
        app.apply_editor_result(&req, Value::Object(map));
        let val = app.config.get("amp.defaultVisibility");
        assert!(val.is_object());
        assert_eq!(val["origin"], Value::String("private".into()));
    }

    #[test]
    fn test_apply_editor_result_array_index() {
        let mut app = test_app();
        app.config.set(
            "amp.permissions",
            Value::Array(vec![Value::Object(serde_json::Map::new())]),
        );
        let req = EditorRequest {
            key: "amp.permissions".to_string(),
            value: Value::Object(serde_json::Map::new()),
            array_index: Some(0),
        };
        let mut edited = serde_json::Map::new();
        edited.insert("tool".into(), Value::String("Bash".into()));
        app.apply_editor_result(&req, Value::Object(edited));
        let arr = app.config.get("amp.permissions");
        assert_eq!(
            arr.as_array().unwrap()[0]["tool"],
            Value::String("Bash".into())
        );
    }

    #[test]
    fn test_unknown_key_returns_editor_request() {
        let mut app = test_app();
        app.selected_section = 4; // Advanced
        app.focus = Focus::Settings;
        let entries = app.current_settings();
        assert!(!entries.is_empty());
        app.selected_setting = 0;
        let req = app.activate_setting();
        assert!(req.is_some());
        assert_eq!(req.unwrap().key, "amp.experimental.modes");
    }

    fn test_app_with_permissions() -> App {
        let mut f = NamedTempFile::new().unwrap();
        write!(
            f,
            r#"{{
    "amp.permissions": [
        {{"tool": "Bash", "decision": "allow"}},
        {{"tool": "Read", "decision": "allow"}},
        {{"tool": "edit_file", "decision": "ask"}}
    ]
}}"#
        )
        .unwrap();
        let config = Config::load(f.path()).unwrap();
        let mut app = App::new(config);
        app.selected_section = 1; // Permissions
        app
    }

    #[test]
    fn test_single_key_item_count() {
        let app = test_app_with_permissions();
        assert_eq!(app.current_section(), Section::Permissions);
        assert_eq!(app.current_item_count(), 3);
    }

    #[test]
    fn test_single_key_navigate_items() {
        let mut app = test_app_with_permissions();
        app.focus = Focus::Settings;
        assert_eq!(app.selected_setting, 0);
        app.move_down();
        assert_eq!(app.selected_setting, 1);
        app.move_down();
        assert_eq!(app.selected_setting, 2);
        app.move_down();
        assert_eq!(app.selected_setting, 2); // stays at last
    }

    #[test]
    fn test_single_key_activate_opens_item() {
        let mut app = test_app_with_permissions();
        app.focus = Focus::Settings;
        app.selected_setting = 1;
        let req = app.activate_setting();
        assert!(req.is_some());
        let req = req.unwrap();
        assert_eq!(req.key, "amp.permissions");
        assert_eq!(req.array_index, Some(1));
        assert_eq!(req.value["tool"], Value::String("Read".into()));
    }

    #[test]
    fn test_single_key_delete_selected_item() {
        let mut app = test_app_with_permissions();
        app.focus = Focus::Settings;
        app.selected_setting = 1; // "Read" item
        app.delete_array_item();
        assert_eq!(app.current_item_count(), 2);
        // The remaining items should be Bash and edit_file
        let arr = app.config.get("amp.permissions");
        let items = arr.as_array().unwrap();
        assert_eq!(items[0]["tool"], Value::String("Bash".into()));
        assert_eq!(items[1]["tool"], Value::String("edit_file".into()));
    }

    #[test]
    fn test_single_key_delete_last_adjusts_selection() {
        let mut app = test_app_with_permissions();
        app.focus = Focus::Settings;
        app.selected_setting = 2; // last item
        app.delete_array_item();
        assert_eq!(app.current_item_count(), 2);
        assert_eq!(app.selected_setting, 1); // adjusted
    }

    #[test]
    fn test_single_key_empty_item_count() {
        let mut app = test_app();
        app.selected_section = 1; // Permissions
        assert_eq!(app.current_item_count(), 0);
    }

    #[test]
    fn test_single_key_reset_clears_array() {
        let mut app = test_app_with_permissions();
        app.focus = Focus::Settings;
        app.selected_setting = 1;
        app.reset_setting();
        assert_eq!(app.current_item_count(), 0);
        assert_eq!(app.selected_setting, 0);
    }
}
