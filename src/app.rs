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

/// Tracks what kind of input the user is currently providing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    /// Not editing anything.
    Normal,
    /// Inline editing a known setting's value (string/number/array item).
    EditingValue,
    /// Entering a key name for a new custom key in Advanced.
    EnteringKeyName,
    /// Selecting a value type for a new custom key.
    SelectingType,
    /// Entering a value for a new custom key (string/number).
    EnteringCustomValue,
    /// Entering the tool name for a new permission rule.
    EnteringPermissionTool,
    /// Selecting the permission level (ask/allow/reject) for a new permission rule.
    SelectingPermissionLevel,
    /// Entering the delegate target program name for a permission rule.
    EnteringDelegateTo,
    /// Confirming whether to open $EDITOR after adding a permission rule.
    ConfirmAdvancedEdit,
    /// Entering the match field (command/url) for a new MCP permission rule.
    EnteringMcpMatchField,
    /// Entering the match value for a new MCP permission rule.
    EnteringMcpMatchValue,
    /// Selecting the MCP permission action (allow/reject).
    SelectingMcpPermissionLevel,
    /// Confirming whether to open $EDITOR after adding an MCP permission rule.
    ConfirmMcpEdit,
    /// Entering the server name for a new MCP server config.
    EnteringMcpServerName,
}

/// Value type choices for custom keys in the Advanced section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomKeyType {
    Boolean,
    String,
    Number,
    Array,
    Object,
}

impl CustomKeyType {
    pub const ALL: &[CustomKeyType] = &[
        CustomKeyType::Boolean,
        CustomKeyType::String,
        CustomKeyType::Number,
        CustomKeyType::Array,
        CustomKeyType::Object,
    ];

    pub fn label(self) -> &'static str {
        match self {
            CustomKeyType::Boolean => "boolean",
            CustomKeyType::String => "string",
            CustomKeyType::Number => "number",
            CustomKeyType::Array => "array",
            CustomKeyType::Object => "object",
        }
    }
}

/// Permission level choices for permission rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionLevel {
    Ask,
    Allow,
    Reject,
    Delegate,
}

impl PermissionLevel {
    pub const ALL: &[PermissionLevel] = &[
        PermissionLevel::Ask,
        PermissionLevel::Allow,
        PermissionLevel::Reject,
        PermissionLevel::Delegate,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PermissionLevel::Ask => "ask",
            PermissionLevel::Allow => "allow",
            PermissionLevel::Reject => "reject",
            PermissionLevel::Delegate => "delegate",
        }
    }
}

/// MCP permission level choices (no delegate option).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpPermissionLevel {
    Allow,
    Reject,
}

impl McpPermissionLevel {
    pub const ALL: &[McpPermissionLevel] = &[McpPermissionLevel::Allow, McpPermissionLevel::Reject];

    pub fn label(self) -> &'static str {
        match self {
            McpPermissionLevel::Allow => "allow",
            McpPermissionLevel::Reject => "reject",
        }
    }
}

/// Which sub-panel has focus in the MCPs split view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpFocus {
    Configs,
    Permissions,
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
    /// For object entries (e.g. amp.mcpServers), the key within the object being edited.
    pub object_key: Option<String>,
}

/// Application state.
pub struct App {
    pub config: Config,
    pub selected_section: usize,
    pub selected_setting: usize,
    pub focus: Focus,
    pub should_quit: bool,
    pub status_message: Option<String>,
    /// Current input mode.
    pub input_mode: InputMode,
    /// Buffer for inline text editing.
    pub edit_buffer: String,
    /// Pending custom key name (used during Advanced add flow).
    pub pending_custom_key: Option<String>,
    /// Selected type index during type selection.
    pub selected_type: usize,
    /// Pending tool name for permission add flow.
    pub pending_permission_tool: Option<String>,
    /// Selected permission level index during permission add flow.
    pub selected_permission_level: usize,
    /// Which sub-panel has focus in the MCPs section.
    pub mcp_focus: McpFocus,
    /// Selected item index in the MCP permissions sub-panel.
    pub selected_mcp_permission: usize,
    /// Selected MCP permission level index during MCP permission add flow.
    pub selected_mcp_permission_level: usize,
    /// Pending match field name for MCP permission add flow (e.g. "command", "url").
    pub pending_mcp_match_field: Option<String>,
    /// Pending match value for MCP permission add flow.
    pub pending_mcp_match_value: Option<String>,
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
            input_mode: InputMode::Normal,
            edit_buffer: String::new(),
            pending_custom_key: None,
            selected_type: 0,
            pending_permission_tool: None,
            selected_permission_level: 0,
            mcp_focus: McpFocus::Configs,
            selected_mcp_permission: 0,
            selected_mcp_permission_level: 0,
            pending_mcp_match_field: None,
            pending_mcp_match_value: None,
        }
    }

    /// Returns whether the app is in any editing/input mode.
    pub fn is_editing(&self) -> bool {
        self.input_mode != InputMode::Normal
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
        } else if self.current_section().is_split_panel() {
            match self.mcp_focus {
                McpFocus::Configs => self.mcp_config_count(),
                McpFocus::Permissions => self.mcp_permission_item_count(),
            }
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

    /// Returns the sorted server names from amp.mcpServers.
    pub fn mcp_server_names(&self) -> Vec<String> {
        self.config
            .get("amp.mcpServers")
            .as_object()
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Returns the number of MCP server config entries.
    pub fn mcp_config_count(&self) -> usize {
        self.mcp_server_names().len()
    }

    /// Returns the number of MCP permission items.
    pub fn mcp_permission_item_count(&self) -> usize {
        self.config
            .get("amp.mcpPermissions")
            .as_array()
            .map_or(0, |a| a.len())
    }

    /// Moves selection up in the current panel.
    pub fn move_up(&mut self) {
        match self.focus {
            Focus::Sidebar => {
                if self.selected_section > 0 {
                    self.selected_section -= 1;
                    self.selected_setting = 0;
                    self.mcp_focus = McpFocus::Configs;
                    self.selected_mcp_permission = 0;
                }
            }
            Focus::Settings => {
                if self.current_section().is_split_panel() {
                    match self.mcp_focus {
                        McpFocus::Configs => {
                            if self.selected_setting > 0 {
                                self.selected_setting -= 1;
                            }
                        }
                        McpFocus::Permissions => {
                            if self.selected_mcp_permission > 0 {
                                self.selected_mcp_permission -= 1;
                            } else {
                                // Move focus to configs panel
                                self.mcp_focus = McpFocus::Configs;
                                let count = self.mcp_config_count();
                                self.selected_setting = if count > 0 { count - 1 } else { 0 };
                            }
                        }
                    }
                } else if self.selected_setting > 0 {
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
                    self.mcp_focus = McpFocus::Configs;
                    self.selected_mcp_permission = 0;
                }
            }
            Focus::Settings => {
                if self.current_section().is_split_panel() {
                    match self.mcp_focus {
                        McpFocus::Configs => {
                            let count = self.mcp_config_count();
                            if count > 0 && self.selected_setting < count - 1 {
                                self.selected_setting += 1;
                            } else {
                                // Move focus to permissions panel
                                self.mcp_focus = McpFocus::Permissions;
                                self.selected_mcp_permission = 0;
                            }
                        }
                        McpFocus::Permissions => {
                            let count = self.mcp_permission_item_count();
                            if count > 0 && self.selected_mcp_permission < count - 1 {
                                self.selected_mcp_permission += 1;
                            }
                        }
                    }
                } else {
                    let count = self.current_item_count();
                    if count > 0 && self.selected_setting < count - 1 {
                        self.selected_setting += 1;
                    }
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

        if self.current_section().is_split_panel() {
            return self.activate_mcp_setting();
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
                    self.input_mode = InputMode::EditingValue;
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
                    object_key: None,
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
                            object_key: None,
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
                    object_key: None,
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
            object_key: None,
        })
    }

    /// Activates the selected item in the MCPs split panel.
    fn activate_mcp_setting(&mut self) -> Option<EditorRequest> {
        match self.mcp_focus {
            McpFocus::Configs => {
                let server_names = self.mcp_server_names();
                let name = server_names.get(self.selected_setting)?;
                let servers = self.config.get("amp.mcpServers");
                let server_config = servers.get(name)?.clone();
                Some(EditorRequest {
                    key: "amp.mcpServers".to_string(),
                    value: server_config,
                    array_index: None,
                    object_key: Some(name.clone()),
                })
            }
            McpFocus::Permissions => {
                let arr = self.config.get("amp.mcpPermissions");
                let items = arr.as_array().cloned().unwrap_or_default();
                let item = items.get(self.selected_mcp_permission)?;
                Some(EditorRequest {
                    key: "amp.mcpPermissions".to_string(),
                    value: item.clone(),
                    array_index: Some(self.selected_mcp_permission),
                    object_key: None,
                })
            }
        }
    }

    /// Forces opening the current setting in `$EDITOR`.
    pub fn force_editor(&self) -> Option<EditorRequest> {
        if self.current_section().is_split_panel() {
            match self.mcp_focus {
                McpFocus::Configs => {
                    let server_names = self.mcp_server_names();
                    let name = server_names.get(self.selected_setting)?;
                    let servers = self.config.get("amp.mcpServers");
                    let server_config = servers.get(name)?.clone();
                    return Some(EditorRequest {
                        key: "amp.mcpServers".to_string(),
                        value: server_config,
                        array_index: None,
                        object_key: Some(name.clone()),
                    });
                }
                McpFocus::Permissions => {
                    let arr = self.config.get("amp.mcpPermissions");
                    let items = arr.as_array().cloned().unwrap_or_default();
                    return items
                        .get(self.selected_mcp_permission)
                        .map(|item| EditorRequest {
                            key: "amp.mcpPermissions".to_string(),
                            value: item.clone(),
                            array_index: Some(self.selected_mcp_permission),
                            object_key: None,
                        });
                }
            }
        }

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
            object_key: None,
        })
    }

    /// Applies the result from an external editor back to the config.
    pub fn apply_editor_result(&mut self, request: &EditorRequest, edited: Value) {
        if let Some(ref obj_key) = request.object_key {
            let mut obj = self
                .config
                .get(&request.key)
                .as_object()
                .cloned()
                .unwrap_or_default();
            obj.insert(obj_key.clone(), edited);
            self.config.set(&request.key, Value::Object(obj));
            self.status_message = Some(format!("Updated {} in {}", obj_key, request.key));
        } else if let Some(idx) = request.array_index {
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
            self.status_message = Some(format!("Updated {}", request.key));
        } else {
            self.config.set(&request.key, edited);
            self.status_message = Some(format!("Updated {}", request.key));
        }
    }

    /// Adds an item to a string array setting (prompts for value via edit buffer).
    pub fn add_array_item(&mut self) {
        if self.current_section() == Section::Advanced {
            self.start_add_custom_key();
            return;
        }

        if self.current_section().is_split_panel() {
            match self.mcp_focus {
                McpFocus::Configs => {
                    self.start_add_mcp_server();
                    return;
                }
                McpFocus::Permissions => {
                    self.start_add_mcp_permission();
                    return;
                }
            }
        }

        let def = self.selected_array_def();
        let Some(def) = def else {
            return;
        };

        match def.setting_type {
            SettingType::ArrayString => {
                self.input_mode = InputMode::EditingValue;
                self.edit_buffer.clear();
            }
            SettingType::ArrayObject => {
                if def.key == "amp.permissions" {
                    self.input_mode = InputMode::EnteringPermissionTool;
                    self.edit_buffer.clear();
                } else {
                    self.input_mode = InputMode::EditingValue;
                    self.edit_buffer.clear();
                }
            }
            _ => {}
        }
    }

    /// Deletes an item from an array setting.
    /// In single-key sections, deletes the selected item; otherwise deletes the last.
    pub fn delete_array_item(&mut self) {
        if self.current_section().is_split_panel() {
            match self.mcp_focus {
                McpFocus::Configs => {
                    self.delete_mcp_config_item();
                    return;
                }
                McpFocus::Permissions => {
                    self.delete_mcp_permission_item();
                    return;
                }
            }
        }

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
        let next_value = options[next_idx];
        if next_value == "Custom" && def.allows_custom {
            self.input_mode = InputMode::EditingValue;
            self.edit_buffer.clear();
        } else {
            self.config
                .set(def.key, Value::String(next_value.to_string()));
        }
    }

    /// Commits the current inline edit.
    pub fn commit_edit(&mut self) {
        if self.input_mode != InputMode::EditingValue {
            return;
        }
        self.input_mode = InputMode::Normal;

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

    /// Starts the "add custom key" flow in the Advanced section.
    pub fn start_add_custom_key(&mut self) {
        if self.current_section() != Section::Advanced {
            return;
        }
        self.input_mode = InputMode::EnteringKeyName;
        self.edit_buffer.clear();
    }

    /// Commits the key name entry and moves to type selection.
    pub fn commit_key_name(&mut self) {
        if self.edit_buffer.trim().is_empty() {
            self.status_message = Some("Key name cannot be empty.".to_string());
            return;
        }
        let key = self.edit_buffer.trim().to_string();
        if self.config.get_raw(&key).is_some() {
            self.status_message = Some(format!("Key '{}' already exists.", key));
            return;
        }
        self.pending_custom_key = Some(key);
        self.edit_buffer.clear();
        self.selected_type = 0;
        self.input_mode = InputMode::SelectingType;
    }

    /// Commits the type selection and either sets the value or transitions to value entry.
    /// Returns an `EditorRequest` if the type requires `$EDITOR`.
    pub fn commit_type_selection(&mut self) -> Option<EditorRequest> {
        let key = self.pending_custom_key.clone()?;
        let chosen = CustomKeyType::ALL[self.selected_type];

        match chosen {
            CustomKeyType::Boolean => {
                self.config.set(&key, Value::Bool(false));
                self.status_message = Some(format!("Added '{}' = false", key));
                self.pending_custom_key = None;
                self.input_mode = InputMode::Normal;
                None
            }
            CustomKeyType::String => {
                self.input_mode = InputMode::EnteringCustomValue;
                self.edit_buffer.clear();
                None
            }
            CustomKeyType::Number => {
                self.input_mode = InputMode::EnteringCustomValue;
                self.edit_buffer.clear();
                None
            }
            CustomKeyType::Array => {
                self.config.set(&key, Value::Array(vec![]));
                self.status_message = Some(format!("Added '{}' = []", key));
                self.pending_custom_key = None;
                self.input_mode = InputMode::Normal;
                None
            }
            CustomKeyType::Object => {
                self.input_mode = InputMode::Normal;
                let req = EditorRequest {
                    key: key.clone(),
                    value: Value::Object(serde_json::Map::new()),
                    array_index: None,
                    object_key: None,
                };
                self.pending_custom_key = None;
                Some(req)
            }
        }
    }

    /// Commits the custom value entry for a pending custom key.
    pub fn commit_custom_value(&mut self) {
        let Some(key) = self.pending_custom_key.take() else {
            self.input_mode = InputMode::Normal;
            return;
        };
        let chosen = CustomKeyType::ALL[self.selected_type];
        match chosen {
            CustomKeyType::String => {
                self.config
                    .set(&key, Value::String(self.edit_buffer.clone()));
                self.status_message = Some(format!("Added '{}'", key));
            }
            CustomKeyType::Number => {
                if let Ok(n) = self.edit_buffer.parse::<i64>() {
                    self.config.set(&key, Value::Number(n.into()));
                    self.status_message = Some(format!("Added '{}'", key));
                } else if let Ok(n) = self.edit_buffer.parse::<f64>() {
                    if let Some(n) = serde_json::Number::from_f64(n) {
                        self.config.set(&key, Value::Number(n));
                        self.status_message = Some(format!("Added '{}'", key));
                    } else {
                        self.status_message = Some("Invalid number.".to_string());
                        self.pending_custom_key = Some(key);
                        return;
                    }
                } else {
                    self.status_message = Some("Invalid number.".to_string());
                    self.pending_custom_key = Some(key);
                    return;
                }
            }
            _ => {}
        }
        self.edit_buffer.clear();
        self.input_mode = InputMode::Normal;
    }

    /// Commits the permission tool name and moves to permission level selection.
    pub fn commit_permission_tool(&mut self) {
        if self.edit_buffer.trim().is_empty() {
            self.status_message = Some("Tool name cannot be empty.".to_string());
            return;
        }
        self.pending_permission_tool = Some(self.edit_buffer.trim().to_string());
        self.edit_buffer.clear();
        self.selected_permission_level = 0;
        self.input_mode = InputMode::SelectingPermissionLevel;
    }

    /// Commits the permission level selection and adds the permission rule.
    /// For `delegate`, transitions to entering the target program name first.
    pub fn commit_permission_level(&mut self) {
        let level = PermissionLevel::ALL[self.selected_permission_level];
        if level == PermissionLevel::Delegate {
            self.input_mode = InputMode::EnteringDelegateTo;
            self.edit_buffer.clear();
            return;
        }

        let Some(tool) = self.pending_permission_tool.take() else {
            self.input_mode = InputMode::Normal;
            return;
        };
        let mut obj = serde_json::Map::new();
        obj.insert("tool".to_string(), Value::String(tool.clone()));
        obj.insert(
            "action".to_string(),
            Value::String(level.label().to_string()),
        );

        let mut arr = self
            .config
            .get("amp.permissions")
            .as_array()
            .cloned()
            .unwrap_or_default();
        arr.push(Value::Object(obj));
        self.config.set("amp.permissions", Value::Array(arr));

        self.status_message = Some(format!("Added permission: {} = {}", tool, level.label()));
        self.input_mode = InputMode::ConfirmAdvancedEdit;
    }

    /// Commits the delegate target and adds the permission rule with the `to` field.
    pub fn commit_delegate_to(&mut self) {
        if self.edit_buffer.trim().is_empty() {
            self.status_message = Some("Program name cannot be empty.".to_string());
            return;
        }
        let to = self.edit_buffer.trim().to_string();

        let Some(tool) = self.pending_permission_tool.take() else {
            self.input_mode = InputMode::Normal;
            return;
        };
        let mut obj = serde_json::Map::new();
        obj.insert("tool".to_string(), Value::String(tool.clone()));
        obj.insert("action".to_string(), Value::String("delegate".to_string()));
        obj.insert("to".to_string(), Value::String(to.clone()));

        let mut arr = self
            .config
            .get("amp.permissions")
            .as_array()
            .cloned()
            .unwrap_or_default();
        arr.push(Value::Object(obj));
        self.config.set("amp.permissions", Value::Array(arr));

        self.status_message = Some(format!("Added permission: {} = delegate to {}", tool, to));
        self.edit_buffer.clear();
        self.input_mode = InputMode::ConfirmAdvancedEdit;
    }

    /// Moves permission level selection up.
    pub fn permission_level_up(&mut self) {
        if self.selected_permission_level > 0 {
            self.selected_permission_level -= 1;
        }
    }

    /// Moves permission level selection down.
    pub fn permission_level_down(&mut self) {
        if self.selected_permission_level < PermissionLevel::ALL.len() - 1 {
            self.selected_permission_level += 1;
        }
    }

    /// Confirms opening $EDITOR for the last-added permission rule.
    /// Returns an `EditorRequest` for the last item in the permissions array.
    pub fn confirm_advanced_edit(&mut self) -> Option<EditorRequest> {
        self.input_mode = InputMode::Normal;
        let arr = self
            .config
            .get("amp.permissions")
            .as_array()
            .cloned()
            .unwrap_or_default();
        let idx = arr.len().checked_sub(1)?;
        Some(EditorRequest {
            key: "amp.permissions".to_string(),
            value: arr[idx].clone(),
            array_index: Some(idx),
            object_key: None,
        })
    }

    /// Declines opening $EDITOR after adding a permission rule.
    pub fn decline_advanced_edit(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    /// Moves type selection up.
    pub fn type_select_up(&mut self) {
        if self.selected_type > 0 {
            self.selected_type -= 1;
        }
    }

    /// Moves type selection down.
    pub fn type_select_down(&mut self) {
        if self.selected_type < CustomKeyType::ALL.len() - 1 {
            self.selected_type += 1;
        }
    }

    /// Cancels the current inline edit.
    pub fn cancel_edit(&mut self) {
        self.input_mode = InputMode::Normal;
        self.edit_buffer.clear();
        self.pending_custom_key = None;
        self.selected_type = 0;
        self.pending_permission_tool = None;
        self.selected_permission_level = 0;
        self.pending_mcp_match_field = None;
        self.pending_mcp_match_value = None;
        self.selected_mcp_permission_level = 0;
    }

    /// Resets the currently selected setting to its default.
    pub fn reset_setting(&mut self) {
        if self.current_section().is_split_panel() {
            match self.mcp_focus {
                McpFocus::Configs => {
                    let server_names = self.mcp_server_names();
                    if let Some(name) = server_names.get(self.selected_setting) {
                        let mut obj = self
                            .config
                            .get("amp.mcpServers")
                            .as_object()
                            .cloned()
                            .unwrap_or_default();
                        obj.remove(name);
                        self.config
                            .set("amp.mcpServers", Value::Object(obj.clone()));
                        self.status_message = Some(format!("Removed server '{}'", name));
                        let count = obj.len();
                        if count > 0 && self.selected_setting >= count {
                            self.selected_setting = count - 1;
                        }
                    }
                }
                McpFocus::Permissions => {
                    self.config.remove("amp.mcpPermissions");
                    self.status_message = Some("Reset amp.mcpPermissions to default".to_string());
                    self.selected_mcp_permission = 0;
                }
            }
            return;
        }

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

    /// Starts the "add MCP server" flow.
    fn start_add_mcp_server(&mut self) {
        self.input_mode = InputMode::EnteringMcpServerName;
        self.edit_buffer.clear();
    }

    /// Commits the server name and opens `$EDITOR` for the new server config.
    pub fn commit_mcp_server_name(&mut self) -> Option<EditorRequest> {
        let name = self.edit_buffer.trim().to_string();
        if name.is_empty() {
            self.status_message = Some("Server name cannot be empty.".to_string());
            return None;
        }
        let servers = self.config.get("amp.mcpServers");
        if servers.get(&name).is_some() {
            self.status_message = Some(format!("Server '{}' already exists.", name));
            return None;
        }
        self.edit_buffer.clear();
        self.input_mode = InputMode::Normal;
        Some(EditorRequest {
            key: "amp.mcpServers".to_string(),
            value: Value::Object(serde_json::Map::new()),
            array_index: None,
            object_key: Some(name),
        })
    }

    /// Deletes the selected MCP server config.
    fn delete_mcp_config_item(&mut self) {
        let server_names = self.mcp_server_names();
        if server_names.is_empty() {
            self.status_message = Some("No servers to delete.".to_string());
            return;
        }
        let idx = self.selected_setting.min(server_names.len() - 1);
        let name = &server_names[idx];
        let mut obj = self
            .config
            .get("amp.mcpServers")
            .as_object()
            .cloned()
            .unwrap_or_default();
        obj.remove(name);
        self.status_message = Some(format!("Removed server '{}'", name));
        self.config
            .set("amp.mcpServers", Value::Object(obj.clone()));
        if !obj.is_empty() && self.selected_setting >= obj.len() {
            self.selected_setting = obj.len() - 1;
        }
    }

    /// Starts the MCP permission add flow.
    fn start_add_mcp_permission(&mut self) {
        self.input_mode = InputMode::EnteringMcpMatchField;
        self.edit_buffer.clear();
    }

    /// Commits the match field name (e.g. "command", "url") for an MCP permission rule.
    pub fn commit_mcp_match_field(&mut self) {
        let field = self.edit_buffer.trim().to_string();
        if field.is_empty() {
            self.status_message = Some("Match field cannot be empty.".to_string());
            return;
        }
        self.pending_mcp_match_field = Some(field);
        self.edit_buffer.clear();
        self.input_mode = InputMode::EnteringMcpMatchValue;
    }

    /// Commits the match value and moves to MCP permission level selection.
    pub fn commit_mcp_match_value(&mut self) {
        if self.edit_buffer.trim().is_empty() {
            self.status_message = Some("Match value cannot be empty.".to_string());
            return;
        }
        self.pending_mcp_match_value = Some(self.edit_buffer.trim().to_string());
        self.edit_buffer.clear();
        self.selected_mcp_permission_level = 0;
        self.input_mode = InputMode::SelectingMcpPermissionLevel;
    }

    /// Commits the MCP permission level and adds the rule.
    pub fn commit_mcp_permission_level(&mut self) {
        let level = McpPermissionLevel::ALL[self.selected_mcp_permission_level];

        let Some(field) = self.pending_mcp_match_field.take() else {
            self.input_mode = InputMode::Normal;
            return;
        };
        let Some(value) = self.pending_mcp_match_value.take() else {
            self.input_mode = InputMode::Normal;
            return;
        };

        let mut matches_obj = serde_json::Map::new();
        matches_obj.insert(field.clone(), Value::String(value.clone()));

        let mut obj = serde_json::Map::new();
        obj.insert("matches".to_string(), Value::Object(matches_obj));
        obj.insert(
            "action".to_string(),
            Value::String(level.label().to_string()),
        );

        let mut arr = self
            .config
            .get("amp.mcpPermissions")
            .as_array()
            .cloned()
            .unwrap_or_default();
        arr.push(Value::Object(obj));
        self.config.set("amp.mcpPermissions", Value::Array(arr));

        self.status_message = Some(format!(
            "Added MCP permission: {field}={value} = {}",
            level.label()
        ));
        self.input_mode = InputMode::ConfirmMcpEdit;
    }

    /// Confirms opening $EDITOR for the last-added MCP permission rule.
    pub fn confirm_mcp_edit(&mut self) -> Option<EditorRequest> {
        self.input_mode = InputMode::Normal;
        let arr = self
            .config
            .get("amp.mcpPermissions")
            .as_array()
            .cloned()
            .unwrap_or_default();
        let idx = arr.len().checked_sub(1)?;
        Some(EditorRequest {
            key: "amp.mcpPermissions".to_string(),
            value: arr[idx].clone(),
            array_index: Some(idx),
            object_key: None,
        })
    }

    /// Declines opening $EDITOR after adding an MCP permission rule.
    pub fn decline_mcp_edit(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    /// Moves MCP permission level selection up.
    pub fn mcp_permission_level_up(&mut self) {
        if self.selected_mcp_permission_level > 0 {
            self.selected_mcp_permission_level -= 1;
        }
    }

    /// Moves MCP permission level selection down.
    pub fn mcp_permission_level_down(&mut self) {
        if self.selected_mcp_permission_level < McpPermissionLevel::ALL.len() - 1 {
            self.selected_mcp_permission_level += 1;
        }
    }

    /// Deletes the selected MCP permission item.
    fn delete_mcp_permission_item(&mut self) {
        let mut arr = self
            .config
            .get("amp.mcpPermissions")
            .as_array()
            .cloned()
            .unwrap_or_default();
        if arr.is_empty() {
            self.status_message = Some("Array is already empty.".to_string());
            return;
        }
        let idx = self.selected_mcp_permission.min(arr.len() - 1);
        arr.remove(idx);
        self.config
            .set("amp.mcpPermissions", Value::Array(arr.clone()));
        self.status_message = Some(format!("Removed MCP permission item {}", idx));
        if !arr.is_empty() && self.selected_mcp_permission >= arr.len() {
            self.selected_mcp_permission = arr.len() - 1;
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
        assert_eq!(app.input_mode, InputMode::Normal);
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
    fn test_cycle_enum_custom_prompts_for_value() {
        let mut app = test_app();
        app.focus = Focus::Settings;
        let entries = app.current_settings();
        let theme_idx = entries
            .iter()
            .position(|e| matches!(e, SettingEntry::Known(d) if d.key == "amp.terminal.theme"))
            .unwrap();
        app.selected_setting = theme_idx;

        // Set theme to "nord" (the option just before "Custom")
        app.config
            .set("amp.terminal.theme", Value::String("nord".to_string()));

        // Cycling from "nord" should land on "Custom" and enter editing mode
        app.activate_setting();
        assert_eq!(app.input_mode, InputMode::EditingValue);
        assert_eq!(app.edit_buffer, "");

        // Typing a custom name and committing should set it
        app.edit_buffer = "my-custom-theme".to_string();
        app.commit_edit();
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(
            app.config.get("amp.terminal.theme"),
            Value::String("my-custom-theme".to_string())
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
        assert!(app.is_editing());
        app.edit_buffer = "my-token".to_string();
        app.commit_edit();
        assert!(!app.is_editing());
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
        assert!(app.is_editing());
        app.edit_buffer = "120".to_string();
        app.commit_edit();
        assert!(!app.is_editing());
        assert_eq!(
            app.config.get("amp.tools.stopTimeout"),
            Value::Number(120.into())
        );
    }

    #[test]
    fn test_inline_edit_cancel() {
        let mut app = test_app();
        app.input_mode = InputMode::EditingValue;
        app.edit_buffer = "something".to_string();
        app.cancel_edit();
        assert!(!app.is_editing());
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
        assert!(app.is_editing());
        app.edit_buffer = "*.rs".to_string();
        app.commit_edit();
        assert!(!app.is_editing());
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
            object_key: None,
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
            object_key: None,
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

    #[test]
    fn test_start_add_custom_key() {
        let mut app = test_app();
        app.selected_section = 4; // Advanced
        app.focus = Focus::Settings;
        app.start_add_custom_key();
        assert_eq!(app.input_mode, InputMode::EnteringKeyName);
        assert!(app.edit_buffer.is_empty());
    }

    #[test]
    fn test_start_add_custom_key_not_advanced() {
        let mut app = test_app();
        app.selected_section = 0; // General
        app.start_add_custom_key();
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_commit_key_name_empty() {
        let mut app = test_app();
        app.selected_section = 4;
        app.input_mode = InputMode::EnteringKeyName;
        app.edit_buffer = "  ".to_string();
        app.commit_key_name();
        assert_eq!(app.input_mode, InputMode::EnteringKeyName);
        assert!(app.status_message.unwrap().contains("empty"));
    }

    #[test]
    fn test_commit_key_name_duplicate() {
        let mut app = test_app();
        app.selected_section = 4;
        app.input_mode = InputMode::EnteringKeyName;
        app.edit_buffer = "amp.showCosts".to_string();
        app.commit_key_name();
        assert_eq!(app.input_mode, InputMode::EnteringKeyName);
        assert!(app.status_message.unwrap().contains("already exists"));
    }

    #[test]
    fn test_commit_key_name_success() {
        let mut app = test_app();
        app.selected_section = 4;
        app.input_mode = InputMode::EnteringKeyName;
        app.edit_buffer = "my.custom.key".to_string();
        app.commit_key_name();
        assert_eq!(app.input_mode, InputMode::SelectingType);
        assert_eq!(app.pending_custom_key.as_deref(), Some("my.custom.key"));
        assert!(app.edit_buffer.is_empty());
    }

    #[test]
    fn test_commit_type_boolean() {
        let mut app = test_app();
        app.pending_custom_key = Some("my.bool.key".to_string());
        app.selected_type = 0; // Boolean
        let req = app.commit_type_selection();
        assert!(req.is_none());
        assert_eq!(app.config.get("my.bool.key"), Value::Bool(false));
        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.pending_custom_key.is_none());
    }

    #[test]
    fn test_commit_type_string_enters_value_mode() {
        let mut app = test_app();
        app.pending_custom_key = Some("my.str.key".to_string());
        app.selected_type = 1; // String
        let req = app.commit_type_selection();
        assert!(req.is_none());
        assert_eq!(app.input_mode, InputMode::EnteringCustomValue);
        assert!(app.pending_custom_key.is_some());
    }

    #[test]
    fn test_commit_type_number_enters_value_mode() {
        let mut app = test_app();
        app.pending_custom_key = Some("my.num.key".to_string());
        app.selected_type = 2; // Number
        let req = app.commit_type_selection();
        assert!(req.is_none());
        assert_eq!(app.input_mode, InputMode::EnteringCustomValue);
    }

    #[test]
    fn test_commit_type_array() {
        let mut app = test_app();
        app.pending_custom_key = Some("my.arr.key".to_string());
        app.selected_type = 3; // Array
        let req = app.commit_type_selection();
        assert!(req.is_none());
        assert_eq!(app.config.get("my.arr.key"), Value::Array(vec![]));
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_commit_type_object_returns_editor_request() {
        let mut app = test_app();
        app.pending_custom_key = Some("my.obj.key".to_string());
        app.selected_type = 4; // Object
        let req = app.commit_type_selection();
        assert!(req.is_some());
        let req = req.unwrap();
        assert_eq!(req.key, "my.obj.key");
        assert!(req.value.is_object());
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_commit_custom_value_string() {
        let mut app = test_app();
        app.pending_custom_key = Some("my.str.key".to_string());
        app.selected_type = 1; // String
        app.input_mode = InputMode::EnteringCustomValue;
        app.edit_buffer = "hello world".to_string();
        app.commit_custom_value();
        assert_eq!(
            app.config.get("my.str.key"),
            Value::String("hello world".into())
        );
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_commit_custom_value_number() {
        let mut app = test_app();
        app.pending_custom_key = Some("my.num.key".to_string());
        app.selected_type = 2; // Number
        app.input_mode = InputMode::EnteringCustomValue;
        app.edit_buffer = "42".to_string();
        app.commit_custom_value();
        assert_eq!(app.config.get("my.num.key"), Value::Number(42.into()));
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_commit_custom_value_invalid_number() {
        let mut app = test_app();
        app.pending_custom_key = Some("my.num.key".to_string());
        app.selected_type = 2; // Number
        app.input_mode = InputMode::EnteringCustomValue;
        app.edit_buffer = "not a number".to_string();
        app.commit_custom_value();
        assert!(app.status_message.unwrap().contains("Invalid"));
        assert!(app.pending_custom_key.is_some());
        assert_eq!(app.input_mode, InputMode::EnteringCustomValue);
    }

    #[test]
    fn test_type_select_navigation() {
        let mut app = test_app();
        app.selected_type = 0;
        app.type_select_up();
        assert_eq!(app.selected_type, 0); // stays at 0
        app.type_select_down();
        assert_eq!(app.selected_type, 1);
        app.type_select_down();
        assert_eq!(app.selected_type, 2);
        // Go to last
        for _ in 0..10 {
            app.type_select_down();
        }
        assert_eq!(app.selected_type, CustomKeyType::ALL.len() - 1);
    }

    #[test]
    fn test_cancel_edit_clears_custom_key_state() {
        let mut app = test_app();
        app.input_mode = InputMode::SelectingType;
        app.pending_custom_key = Some("my.key".to_string());
        app.selected_type = 2;
        app.cancel_edit();
        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.pending_custom_key.is_none());
        assert_eq!(app.selected_type, 0);
    }

    #[test]
    fn test_add_custom_key_full_flow_string() {
        let mut app = test_app();
        app.selected_section = 4; // Advanced
        app.focus = Focus::Settings;

        // Step 1: start
        app.add_array_item();
        assert_eq!(app.input_mode, InputMode::EnteringKeyName);

        // Step 2: enter key name
        app.edit_buffer = "my.custom.setting".to_string();
        app.commit_key_name();
        assert_eq!(app.input_mode, InputMode::SelectingType);

        // Step 3: select string type
        app.selected_type = 1; // String
        app.commit_type_selection();
        assert_eq!(app.input_mode, InputMode::EnteringCustomValue);

        // Step 4: enter value
        app.edit_buffer = "my value".to_string();
        app.commit_custom_value();
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(
            app.config.get("my.custom.setting"),
            Value::String("my value".into())
        );
    }

    #[test]
    fn test_permission_add_starts_tool_prompt() {
        let mut app = test_app();
        app.selected_section = 1; // Permissions
        app.focus = Focus::Settings;
        app.add_array_item();
        assert_eq!(app.input_mode, InputMode::EnteringPermissionTool);
        assert!(app.edit_buffer.is_empty());
    }

    #[test]
    fn test_permission_tool_empty_rejected() {
        let mut app = test_app();
        app.input_mode = InputMode::EnteringPermissionTool;
        app.edit_buffer = "  ".to_string();
        app.commit_permission_tool();
        assert_eq!(app.input_mode, InputMode::EnteringPermissionTool);
        assert!(app.status_message.unwrap().contains("empty"));
    }

    #[test]
    fn test_permission_tool_moves_to_level_select() {
        let mut app = test_app();
        app.input_mode = InputMode::EnteringPermissionTool;
        app.edit_buffer = "Bash".to_string();
        app.commit_permission_tool();
        assert_eq!(app.input_mode, InputMode::SelectingPermissionLevel);
        assert_eq!(app.pending_permission_tool.as_deref(), Some("Bash"));
        assert_eq!(app.selected_permission_level, 0);
    }

    #[test]
    fn test_permission_level_navigation() {
        let mut app = test_app();
        app.selected_permission_level = 0;
        app.permission_level_up();
        assert_eq!(app.selected_permission_level, 0);
        app.permission_level_down();
        assert_eq!(app.selected_permission_level, 1);
        app.permission_level_down();
        assert_eq!(app.selected_permission_level, 2);
        app.permission_level_down();
        assert_eq!(app.selected_permission_level, 3); // delegate
        app.permission_level_down();
        assert_eq!(app.selected_permission_level, 3); // stays at last
    }

    #[test]
    fn test_permission_commit_adds_rule() {
        let mut app = test_app();
        app.pending_permission_tool = Some("Bash".to_string());
        app.selected_permission_level = 1; // allow
        app.commit_permission_level();
        assert_eq!(app.input_mode, InputMode::ConfirmAdvancedEdit);
        assert!(app.pending_permission_tool.is_none());

        let arr = app.config.get("amp.permissions");
        let items = arr.as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["tool"], Value::String("Bash".into()));
        assert_eq!(items[0]["action"], Value::String("allow".into()));
    }

    #[test]
    fn test_permission_full_flow() {
        let mut app = test_app();
        app.selected_section = 1; // Permissions
        app.focus = Focus::Settings;

        // Step 1: press 'a' to start
        app.add_array_item();
        assert_eq!(app.input_mode, InputMode::EnteringPermissionTool);

        // Step 2: enter tool name
        app.edit_buffer = "Read".to_string();
        app.commit_permission_tool();
        assert_eq!(app.input_mode, InputMode::SelectingPermissionLevel);

        // Step 3: select "reject" (index 2)
        app.permission_level_down();
        app.permission_level_down();
        assert_eq!(app.selected_permission_level, 2);
        app.commit_permission_level();

        assert_eq!(app.input_mode, InputMode::ConfirmAdvancedEdit);
        let arr = app.config.get("amp.permissions");
        let items = arr.as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["tool"], Value::String("Read".into()));
        assert_eq!(items[0]["action"], Value::String("reject".into()));
    }

    #[test]
    fn test_cancel_permission_clears_state() {
        let mut app = test_app();
        app.input_mode = InputMode::SelectingPermissionLevel;
        app.pending_permission_tool = Some("Bash".to_string());
        app.selected_permission_level = 1;
        app.cancel_edit();
        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.pending_permission_tool.is_none());
        assert_eq!(app.selected_permission_level, 0);
    }

    #[test]
    fn test_confirm_advanced_edit_returns_editor_request() {
        let mut app = test_app();
        // Add a permission rule first
        app.pending_permission_tool = Some("Bash".to_string());
        app.selected_permission_level = 0; // ask
        app.commit_permission_level();
        assert_eq!(app.input_mode, InputMode::ConfirmAdvancedEdit);

        let req = app.confirm_advanced_edit();
        assert!(req.is_some());
        let req = req.unwrap();
        assert_eq!(req.key, "amp.permissions");
        assert_eq!(req.array_index, Some(0));
        assert_eq!(req.value["tool"], Value::String("Bash".into()));
        assert_eq!(req.value["action"], Value::String("ask".into()));
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_decline_advanced_edit_returns_to_normal() {
        let mut app = test_app();
        app.input_mode = InputMode::ConfirmAdvancedEdit;
        app.decline_advanced_edit();
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_permission_full_flow_with_decline() {
        let mut app = test_app();
        app.selected_section = 1; // Permissions
        app.focus = Focus::Settings;

        app.add_array_item();
        app.edit_buffer = "Bash".to_string();
        app.commit_permission_tool();
        app.commit_permission_level(); // defaults to "ask"
        assert_eq!(app.input_mode, InputMode::ConfirmAdvancedEdit);

        app.decline_advanced_edit();
        assert_eq!(app.input_mode, InputMode::Normal);

        let arr = app.config.get("amp.permissions");
        let items = arr.as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["tool"], Value::String("Bash".into()));
    }

    #[test]
    fn test_delegate_level_prompts_for_to() {
        let mut app = test_app();
        app.pending_permission_tool = Some("Bash".to_string());
        app.selected_permission_level = 3; // Delegate
        app.commit_permission_level();
        assert_eq!(app.input_mode, InputMode::EnteringDelegateTo);
        assert!(app.pending_permission_tool.is_some());
    }

    #[test]
    fn test_delegate_to_empty_rejected() {
        let mut app = test_app();
        app.input_mode = InputMode::EnteringDelegateTo;
        app.pending_permission_tool = Some("Bash".to_string());
        app.edit_buffer = "  ".to_string();
        app.commit_delegate_to();
        assert_eq!(app.input_mode, InputMode::EnteringDelegateTo);
        assert!(app.status_message.unwrap().contains("empty"));
    }

    #[test]
    fn test_delegate_full_flow() {
        let mut app = test_app();
        app.selected_section = 1; // Permissions
        app.focus = Focus::Settings;

        app.add_array_item();
        app.edit_buffer = "*".to_string();
        app.commit_permission_tool();

        // Select delegate (index 3)
        app.selected_permission_level = 3;
        app.commit_permission_level();
        assert_eq!(app.input_mode, InputMode::EnteringDelegateTo);

        app.edit_buffer = "my-permission-helper".to_string();
        app.commit_delegate_to();
        assert_eq!(app.input_mode, InputMode::ConfirmAdvancedEdit);

        app.decline_advanced_edit();
        assert_eq!(app.input_mode, InputMode::Normal);

        let arr = app.config.get("amp.permissions");
        let items = arr.as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["tool"], Value::String("*".into()));
        assert_eq!(items[0]["action"], Value::String("delegate".into()));
        assert_eq!(items[0]["to"], Value::String("my-permission-helper".into()));
    }

    fn test_app_with_mcp_permissions() -> App {
        let mut f = NamedTempFile::new().unwrap();
        write!(
            f,
            r#"{{
    "amp.mcpServers": {{"test-server": {{"command": "npx"}}}},
    "amp.mcpPermissions": [
        {{"matches": {{"command": "npx"}}, "action": "allow"}},
        {{"matches": {{"url": "https://evil.com"}}, "action": "reject"}}
    ]
}}"#
        )
        .unwrap();
        let config = Config::load(f.path()).unwrap();
        let mut app = App::new(config);
        app.selected_section = 3; // MCPs
        app
    }

    #[test]
    fn test_mcp_split_initial_focus() {
        let app = test_app_with_mcp_permissions();
        assert_eq!(app.current_section(), Section::Mcps);
        assert_eq!(app.mcp_focus, McpFocus::Configs);
        assert_eq!(app.selected_mcp_permission, 0);
    }

    #[test]
    fn test_mcp_server_names() {
        let app = test_app_with_mcp_permissions();
        let names = app.mcp_server_names();
        assert_eq!(names, vec!["test-server"]);
    }

    #[test]
    fn test_mcp_config_count() {
        let app = test_app_with_mcp_permissions();
        assert_eq!(app.mcp_config_count(), 1);
    }

    #[test]
    fn test_mcp_navigate_configs_to_permissions() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        assert_eq!(app.mcp_focus, McpFocus::Configs);

        // Move down past configs (only 1 entry) should go to permissions
        app.move_down();
        assert_eq!(app.mcp_focus, McpFocus::Permissions);
        assert_eq!(app.selected_mcp_permission, 0);
    }

    #[test]
    fn test_mcp_navigate_permissions_to_configs() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Permissions;
        app.selected_mcp_permission = 0;

        // Move up from top of permissions should go back to configs
        app.move_up();
        assert_eq!(app.mcp_focus, McpFocus::Configs);
    }

    #[test]
    fn test_mcp_navigate_within_permissions() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Permissions;
        app.selected_mcp_permission = 0;

        app.move_down();
        assert_eq!(app.selected_mcp_permission, 1);
        app.move_down();
        assert_eq!(app.selected_mcp_permission, 1); // stays at last

        app.move_up();
        assert_eq!(app.selected_mcp_permission, 0);
    }

    #[test]
    fn test_mcp_permission_item_count() {
        let app = test_app_with_mcp_permissions();
        assert_eq!(app.mcp_permission_item_count(), 2);
    }

    #[test]
    fn test_mcp_activate_config_opens_editor() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Configs;
        app.selected_setting = 0;

        let req = app.activate_setting();
        assert!(req.is_some());
        let req = req.unwrap();
        assert_eq!(req.key, "amp.mcpServers");
        assert_eq!(req.object_key.as_deref(), Some("test-server"));
        assert!(req.array_index.is_none());
        assert_eq!(req.value["command"], Value::String("npx".into()));
    }

    #[test]
    fn test_mcp_activate_permission_opens_item() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Permissions;
        app.selected_mcp_permission = 1;

        let req = app.activate_setting();
        assert!(req.is_some());
        let req = req.unwrap();
        assert_eq!(req.key, "amp.mcpPermissions");
        assert_eq!(req.array_index, Some(1));
        assert_eq!(req.value["action"], Value::String("reject".into()));
    }

    #[test]
    fn test_mcp_permission_add_starts_match_field() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Permissions;
        app.add_array_item();
        assert_eq!(app.input_mode, InputMode::EnteringMcpMatchField);
    }

    #[test]
    fn test_mcp_match_field_empty_rejected() {
        let mut app = test_app_with_mcp_permissions();
        app.input_mode = InputMode::EnteringMcpMatchField;
        app.edit_buffer = "  ".to_string();
        app.commit_mcp_match_field();
        assert_eq!(app.input_mode, InputMode::EnteringMcpMatchField);
        assert!(app.status_message.unwrap().contains("empty"));
    }

    #[test]
    fn test_mcp_match_field_moves_to_value() {
        let mut app = test_app_with_mcp_permissions();
        app.input_mode = InputMode::EnteringMcpMatchField;
        app.edit_buffer = "command".to_string();
        app.commit_mcp_match_field();
        assert_eq!(app.input_mode, InputMode::EnteringMcpMatchValue);
        assert_eq!(app.pending_mcp_match_field.as_deref(), Some("command"));
    }

    #[test]
    fn test_mcp_match_value_empty_rejected() {
        let mut app = test_app_with_mcp_permissions();
        app.input_mode = InputMode::EnteringMcpMatchValue;
        app.pending_mcp_match_field = Some("command".to_string());
        app.edit_buffer = "  ".to_string();
        app.commit_mcp_match_value();
        assert_eq!(app.input_mode, InputMode::EnteringMcpMatchValue);
        assert!(app.status_message.unwrap().contains("empty"));
    }

    #[test]
    fn test_mcp_match_value_moves_to_level_select() {
        let mut app = test_app_with_mcp_permissions();
        app.input_mode = InputMode::EnteringMcpMatchValue;
        app.pending_mcp_match_field = Some("url".to_string());
        app.edit_buffer = "https://example.com".to_string();
        app.commit_mcp_match_value();
        assert_eq!(app.input_mode, InputMode::SelectingMcpPermissionLevel);
        assert_eq!(
            app.pending_mcp_match_value.as_deref(),
            Some("https://example.com")
        );
    }

    #[test]
    fn test_mcp_permission_level_navigation() {
        let mut app = test_app_with_mcp_permissions();
        app.selected_mcp_permission_level = 0;
        app.mcp_permission_level_up();
        assert_eq!(app.selected_mcp_permission_level, 0); // stays at 0
        app.mcp_permission_level_down();
        assert_eq!(app.selected_mcp_permission_level, 1);
        app.mcp_permission_level_down();
        assert_eq!(app.selected_mcp_permission_level, 1); // stays at last (only 2 options)
    }

    #[test]
    fn test_mcp_permission_commit_adds_rule() {
        let mut app = test_app();
        app.pending_mcp_match_field = Some("command".to_string());
        app.pending_mcp_match_value = Some("npx".to_string());
        app.selected_mcp_permission_level = 0; // allow
        app.commit_mcp_permission_level();
        assert_eq!(app.input_mode, InputMode::ConfirmMcpEdit);

        let arr = app.config.get("amp.mcpPermissions");
        let items = arr.as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0]["matches"],
            Value::Object({
                let mut m = serde_json::Map::new();
                m.insert("command".into(), Value::String("npx".into()));
                m
            })
        );
        assert_eq!(items[0]["action"], Value::String("allow".into()));
    }

    #[test]
    fn test_mcp_permission_full_flow() {
        let mut app = test_app();
        app.selected_section = 3; // MCPs
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Permissions;

        // Step 1: start add
        app.add_array_item();
        assert_eq!(app.input_mode, InputMode::EnteringMcpMatchField);

        // Step 2: enter match field
        app.edit_buffer = "url".to_string();
        app.commit_mcp_match_field();
        assert_eq!(app.input_mode, InputMode::EnteringMcpMatchValue);

        // Step 3: enter match value
        app.edit_buffer = "https://evil.com/*".to_string();
        app.commit_mcp_match_value();
        assert_eq!(app.input_mode, InputMode::SelectingMcpPermissionLevel);

        // Step 4: select reject (index 1)
        app.mcp_permission_level_down();
        assert_eq!(app.selected_mcp_permission_level, 1);
        app.commit_mcp_permission_level();
        assert_eq!(app.input_mode, InputMode::ConfirmMcpEdit);

        // Step 5: decline editor
        app.decline_mcp_edit();
        assert_eq!(app.input_mode, InputMode::Normal);

        let arr = app.config.get("amp.mcpPermissions");
        let items = arr.as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["action"], Value::String("reject".into()));
    }

    #[test]
    fn test_mcp_confirm_edit_returns_editor_request() {
        let mut app = test_app();
        app.pending_mcp_match_field = Some("command".to_string());
        app.pending_mcp_match_value = Some("npx".to_string());
        app.selected_mcp_permission_level = 0;
        app.commit_mcp_permission_level();
        assert_eq!(app.input_mode, InputMode::ConfirmMcpEdit);

        let req = app.confirm_mcp_edit();
        assert!(req.is_some());
        let req = req.unwrap();
        assert_eq!(req.key, "amp.mcpPermissions");
        assert_eq!(req.array_index, Some(0));
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_mcp_delete_permission_item() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Permissions;
        app.selected_mcp_permission = 0;

        app.delete_array_item();
        assert_eq!(app.mcp_permission_item_count(), 1);
        let arr = app.config.get("amp.mcpPermissions");
        let items = arr.as_array().unwrap();
        assert_eq!(items[0]["action"], Value::String("reject".into()));
    }

    #[test]
    fn test_mcp_delete_last_adjusts_selection() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Permissions;
        app.selected_mcp_permission = 1; // last item

        app.delete_array_item();
        assert_eq!(app.mcp_permission_item_count(), 1);
        assert_eq!(app.selected_mcp_permission, 0);
    }

    #[test]
    fn test_mcp_reset_permissions() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Permissions;

        app.reset_setting();
        assert_eq!(app.mcp_permission_item_count(), 0);
        assert_eq!(app.selected_mcp_permission, 0);
    }

    #[test]
    fn test_mcp_reset_configs_deletes_server() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Configs;
        app.selected_setting = 0;

        app.reset_setting();
        let val = app.config.get("amp.mcpServers");
        assert!(val.as_object().unwrap().is_empty());
        assert!(app.status_message.unwrap().contains("Removed server"));
    }

    #[test]
    fn test_mcp_force_editor_configs() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Configs;
        app.selected_setting = 0;

        let req = app.force_editor();
        assert!(req.is_some());
        let req = req.unwrap();
        assert_eq!(req.key, "amp.mcpServers");
        assert_eq!(req.object_key.as_deref(), Some("test-server"));
        assert!(req.array_index.is_none());
        assert_eq!(req.value["command"], Value::String("npx".into()));
    }

    #[test]
    fn test_mcp_force_editor_permissions() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Permissions;
        app.selected_mcp_permission = 1;

        let req = app.force_editor();
        assert!(req.is_some());
        let req = req.unwrap();
        assert_eq!(req.key, "amp.mcpPermissions");
        assert_eq!(req.array_index, Some(1));
    }

    #[test]
    fn test_mcp_add_server_starts_name_entry() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Configs;
        app.add_array_item();
        assert_eq!(app.input_mode, InputMode::EnteringMcpServerName);
        assert!(app.edit_buffer.is_empty());
    }

    #[test]
    fn test_mcp_server_name_empty_rejected() {
        let mut app = test_app_with_mcp_permissions();
        app.input_mode = InputMode::EnteringMcpServerName;
        app.edit_buffer = "  ".to_string();
        let req = app.commit_mcp_server_name();
        assert!(req.is_none());
        assert_eq!(app.input_mode, InputMode::EnteringMcpServerName);
        assert!(app.status_message.unwrap().contains("empty"));
    }

    #[test]
    fn test_mcp_server_name_duplicate_rejected() {
        let mut app = test_app_with_mcp_permissions();
        app.input_mode = InputMode::EnteringMcpServerName;
        app.edit_buffer = "test-server".to_string();
        let req = app.commit_mcp_server_name();
        assert!(req.is_none());
        assert_eq!(app.input_mode, InputMode::EnteringMcpServerName);
        assert!(app.status_message.unwrap().contains("already exists"));
    }

    #[test]
    fn test_mcp_server_name_success_returns_editor_request() {
        let mut app = test_app_with_mcp_permissions();
        app.input_mode = InputMode::EnteringMcpServerName;
        app.edit_buffer = "new-server".to_string();
        let req = app.commit_mcp_server_name();
        assert!(req.is_some());
        let req = req.unwrap();
        assert_eq!(req.key, "amp.mcpServers");
        assert_eq!(req.object_key.as_deref(), Some("new-server"));
        assert!(req.value.is_object());
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_mcp_delete_config_item() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Configs;
        app.selected_setting = 0;

        app.delete_array_item();
        assert_eq!(app.mcp_config_count(), 0);
        assert!(app.status_message.unwrap().contains("Removed server"));
    }

    #[test]
    fn test_mcp_delete_config_empty() {
        let mut app = test_app();
        app.selected_section = 3; // MCPs
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Configs;

        app.delete_array_item();
        assert!(app.status_message.unwrap().contains("No servers"));
    }

    #[test]
    fn test_mcp_apply_editor_result_with_object_key() {
        let mut app = test_app_with_mcp_permissions();
        let req = EditorRequest {
            key: "amp.mcpServers".to_string(),
            value: Value::Object(serde_json::Map::new()),
            array_index: None,
            object_key: Some("test-server".to_string()),
        };
        let mut edited = serde_json::Map::new();
        edited.insert("command".into(), Value::String("node".into()));
        edited.insert(
            "args".into(),
            Value::Array(vec![Value::String("server.js".into())]),
        );
        app.apply_editor_result(&req, Value::Object(edited));
        let servers = app.config.get("amp.mcpServers");
        let server = servers.get("test-server").unwrap();
        assert_eq!(server["command"], Value::String("node".into()));
    }

    #[test]
    fn test_mcp_apply_editor_result_new_server() {
        let mut app = test_app_with_mcp_permissions();
        let req = EditorRequest {
            key: "amp.mcpServers".to_string(),
            value: Value::Object(serde_json::Map::new()),
            array_index: None,
            object_key: Some("brand-new".to_string()),
        };
        let mut edited = serde_json::Map::new();
        edited.insert("url".into(), Value::String("https://example.com".into()));
        app.apply_editor_result(&req, Value::Object(edited));
        let servers = app.config.get("amp.mcpServers");
        assert!(servers.get("brand-new").is_some());
        assert_eq!(app.mcp_config_count(), 2);
    }

    #[test]
    fn test_mcp_cancel_edit_clears_state() {
        let mut app = test_app();
        app.input_mode = InputMode::SelectingMcpPermissionLevel;
        app.pending_mcp_match_field = Some("command".to_string());
        app.pending_mcp_match_value = Some("npx".to_string());
        app.selected_mcp_permission_level = 1;
        app.cancel_edit();
        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.pending_mcp_match_field.is_none());
        assert!(app.pending_mcp_match_value.is_none());
        assert_eq!(app.selected_mcp_permission_level, 0);
    }

    #[test]
    fn test_mcp_section_change_resets_mcp_state() {
        let mut app = test_app_with_mcp_permissions();
        app.focus = Focus::Settings;
        app.mcp_focus = McpFocus::Permissions;
        app.selected_mcp_permission = 1;

        // Switch to sidebar and move to different section
        app.focus = Focus::Sidebar;
        app.move_down(); // MCPs -> Advanced
        assert_eq!(app.mcp_focus, McpFocus::Configs);
        assert_eq!(app.selected_mcp_permission, 0);
    }
}
