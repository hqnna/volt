//! Setting definitions and schema for known Amp settings.

use serde_json::Value;

/// The type of a setting value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingType {
    Boolean,
    String,
    Number,
    StringEnum,
    ArrayString,
    ArrayObject,
    Object,
}

/// Definition of a known Amp setting.
#[derive(Debug, Clone)]
pub struct SettingDef {
    pub key: &'static str,
    pub setting_type: SettingType,
    pub default: Value,
    /// For enum types, the list of valid options.
    pub enum_options: Option<&'static [&'static str]>,
    /// Whether the user may enter a custom value beyond the enum options.
    pub allows_custom: bool,
}

/// Which section a setting belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Section {
    General,
    Permissions,
    Tools,
    Mcps,
    Advanced,
}

impl Section {
    pub const ALL: &[Section] = &[
        Section::General,
        Section::Permissions,
        Section::Tools,
        Section::Mcps,
        Section::Advanced,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Section::General => "General",
            Section::Permissions => "Permissions",
            Section::Tools => "Tools",
            Section::Mcps => "MCPs",
            Section::Advanced => "Advanced",
        }
    }

    /// Returns whether this section has exactly one setting (rendered as a full editor).
    pub fn is_single_key(self) -> bool {
        matches!(self, Section::Permissions)
    }

    /// Returns whether this section uses a split panel (top/bottom) layout.
    pub fn is_split_panel(self) -> bool {
        matches!(self, Section::Mcps)
    }
}

/// Theme options for `amp.terminal.theme`.
const THEME_OPTIONS: &[&str] = &[
    "terminal",
    "dark",
    "light",
    "catppuccin-mocha",
    "solarized-dark",
    "solarized-light",
    "gruvbox-dark-hard",
    "nord",
    "Custom",
];

/// Node spawn load profile options.
const LOAD_PROFILE_OPTIONS: &[&str] = &["always", "never", "daily"];

/// Update mode options.
const UPDATE_MODE_OPTIONS: &[&str] = &["auto", "warn", "disabled"];

/// Deep reasoning effort options.
const DEEP_REASONING_OPTIONS: &[&str] = &["medium", "high", "xhigh"];

/// All known Amp settings with their definitions.
pub fn known_settings() -> Vec<SettingDef> {
    vec![
        // General
        SettingDef {
            key: "amp.anthropic.thinking.enabled",
            setting_type: SettingType::Boolean,
            default: Value::Bool(true),
            enum_options: None,
            allows_custom: false,
        },
        SettingDef {
            key: "amp.showCosts",
            setting_type: SettingType::Boolean,
            default: Value::Bool(true),
            enum_options: None,
            allows_custom: false,
        },
        SettingDef {
            key: "amp.notifications.enabled",
            setting_type: SettingType::Boolean,
            default: Value::Bool(true),
            enum_options: None,
            allows_custom: false,
        },
        SettingDef {
            key: "amp.git.commit.ampThread.enabled",
            setting_type: SettingType::Boolean,
            default: Value::Bool(true),
            enum_options: None,
            allows_custom: false,
        },
        SettingDef {
            key: "amp.git.commit.coauthor.enabled",
            setting_type: SettingType::Boolean,
            default: Value::Bool(true),
            enum_options: None,
            allows_custom: false,
        },
        SettingDef {
            key: "amp.tab.clipboard.enabled",
            setting_type: SettingType::Boolean,
            default: Value::Bool(true),
            enum_options: None,
            allows_custom: false,
        },
        SettingDef {
            key: "amp.bitbucketToken",
            setting_type: SettingType::String,
            default: Value::String(String::new()),
            enum_options: None,
            allows_custom: false,
        },
        SettingDef {
            key: "amp.skills.path",
            setting_type: SettingType::String,
            default: Value::String(String::new()),
            enum_options: None,
            allows_custom: false,
        },
        SettingDef {
            key: "amp.terminal.theme",
            setting_type: SettingType::StringEnum,
            default: Value::String(String::new()),
            enum_options: Some(THEME_OPTIONS),
            allows_custom: true,
        },
        SettingDef {
            key: "amp.terminal.commands.nodeSpawn.loadProfile",
            setting_type: SettingType::StringEnum,
            default: Value::String(String::new()),
            enum_options: Some(LOAD_PROFILE_OPTIONS),
            allows_custom: false,
        },
        SettingDef {
            key: "amp.updates.mode",
            setting_type: SettingType::StringEnum,
            default: Value::String(String::new()),
            enum_options: Some(UPDATE_MODE_OPTIONS),
            allows_custom: false,
        },
        SettingDef {
            key: "amp.internal.deepReasoningEffort",
            setting_type: SettingType::StringEnum,
            default: Value::String(String::new()),
            enum_options: Some(DEEP_REASONING_OPTIONS),
            allows_custom: false,
        },
        SettingDef {
            key: "amp.defaultVisibility",
            setting_type: SettingType::Object,
            default: Value::Object(serde_json::Map::new()),
            enum_options: None,
            allows_custom: false,
        },
        SettingDef {
            key: "amp.fuzzy.alwaysIncludePaths",
            setting_type: SettingType::ArrayString,
            default: Value::Array(vec![]),
            enum_options: None,
            allows_custom: false,
        },
        // Permissions
        SettingDef {
            key: "amp.permissions",
            setting_type: SettingType::ArrayObject,
            default: Value::Array(vec![]),
            enum_options: None,
            allows_custom: false,
        },
        // Tools
        SettingDef {
            key: "amp.tools.disable",
            setting_type: SettingType::ArrayString,
            default: Value::Array(vec![]),
            enum_options: None,
            allows_custom: false,
        },
        SettingDef {
            key: "amp.tools.stopTimeout",
            setting_type: SettingType::Number,
            default: Value::Number(serde_json::Number::from(300)),
            enum_options: None,
            allows_custom: false,
        },
        // MCPs
        SettingDef {
            key: "amp.mcpServers",
            setting_type: SettingType::Object,
            default: Value::Object(serde_json::Map::new()),
            enum_options: None,
            allows_custom: false,
        },
        SettingDef {
            key: "amp.mcpPermissions",
            setting_type: SettingType::ArrayObject,
            default: Value::Array(vec![]),
            enum_options: None,
            allows_custom: false,
        },
    ]
}

/// Returns the section for a known setting key.
pub fn section_for_key(key: &str) -> Option<Section> {
    match key {
        "amp.permissions" => Some(Section::Permissions),
        "amp.tools.disable" | "amp.tools.stopTimeout" => Some(Section::Tools),
        "amp.mcpServers" | "amp.mcpPermissions" => Some(Section::Mcps),
        k if known_settings().iter().any(|s| s.key == k) => Some(Section::General),
        _ => None,
    }
}

/// Returns the setting definition for a known key.
pub fn get_setting_def(key: &str) -> Option<SettingDef> {
    known_settings().into_iter().find(|s| s.key == key)
}

/// Returns all known setting keys for a given section.
pub fn settings_for_section(section: Section) -> Vec<SettingDef> {
    known_settings()
        .into_iter()
        .filter(|s| section_for_key(s.key) == Some(section))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_labels() {
        assert_eq!(Section::General.label(), "General");
        assert_eq!(Section::Permissions.label(), "Permissions");
        assert_eq!(Section::Tools.label(), "Tools");
        assert_eq!(Section::Mcps.label(), "MCPs");
        assert_eq!(Section::Advanced.label(), "Advanced");
    }

    #[test]
    fn test_section_for_known_keys() {
        assert_eq!(section_for_key("amp.showCosts"), Some(Section::General));
        assert_eq!(
            section_for_key("amp.permissions"),
            Some(Section::Permissions)
        );
        assert_eq!(section_for_key("amp.tools.disable"), Some(Section::Tools));
        assert_eq!(section_for_key("amp.mcpServers"), Some(Section::Mcps));
    }

    #[test]
    fn test_section_for_unknown_key() {
        assert_eq!(section_for_key("amp.experimental.modes"), None);
        assert_eq!(section_for_key("some.random.key"), None);
    }

    #[test]
    fn test_get_setting_def() {
        let def = get_setting_def("amp.showCosts").unwrap();
        assert_eq!(def.key, "amp.showCosts");
        assert_eq!(def.setting_type, SettingType::Boolean);
        assert_eq!(def.default, Value::Bool(true));

        assert!(get_setting_def("nonexistent").is_none());
    }

    #[test]
    fn test_settings_for_section() {
        let general = settings_for_section(Section::General);
        assert!(general.iter().any(|s| s.key == "amp.showCosts"));
        assert!(general.iter().all(|s| s.key != "amp.permissions"));

        let permissions = settings_for_section(Section::Permissions);
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0].key, "amp.permissions");

        let tools = settings_for_section(Section::Tools);
        assert_eq!(tools.len(), 2);

        let mcps = settings_for_section(Section::Mcps);
        assert_eq!(mcps.len(), 2);
    }

    #[test]
    fn test_all_sections_covered() {
        for section in Section::ALL {
            if *section != Section::Advanced {
                assert!(
                    !settings_for_section(*section).is_empty(),
                    "Section {:?} has no settings",
                    section
                );
            }
        }
    }

    #[test]
    fn test_enum_options() {
        let theme = get_setting_def("amp.terminal.theme").unwrap();
        assert_eq!(theme.setting_type, SettingType::StringEnum);
        assert!(theme.enum_options.unwrap().contains(&"terminal"));
        assert!(theme.enum_options.unwrap().contains(&"Custom"));

        let update = get_setting_def("amp.updates.mode").unwrap();
        assert!(update.enum_options.unwrap().contains(&"auto"));
    }

    #[test]
    fn test_no_duplicate_keys() {
        let settings = known_settings();
        let mut keys: Vec<&str> = settings.iter().map(|s| s.key).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), settings.len(), "Duplicate keys found");
    }

    #[test]
    fn test_is_single_key() {
        assert!(Section::Permissions.is_single_key());
        assert!(!Section::General.is_single_key());
        assert!(!Section::Tools.is_single_key());
        assert!(!Section::Mcps.is_single_key());
        assert!(!Section::Advanced.is_single_key());
    }

    #[test]
    fn test_is_split_panel() {
        assert!(Section::Mcps.is_split_panel());
        assert!(!Section::General.is_split_panel());
        assert!(!Section::Permissions.is_split_panel());
        assert!(!Section::Tools.is_split_panel());
        assert!(!Section::Advanced.is_split_panel());
    }
}
