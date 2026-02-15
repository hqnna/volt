//! Configuration file loading and saving for Amp's settings.json.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{Map, Value};

use crate::settings::{self, SettingType};

/// Represents the loaded configuration state.
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the settings.json file.
    path: PathBuf,
    /// All setting values (known + unknown), keyed by setting name.
    values: BTreeMap<String, Value>,
    /// Whether values have been modified since last save/load.
    dirty: bool,
}

impl Config {
    /// Loads settings from the given path, or creates an empty config if the file
    /// doesn't exist.
    pub fn load(path: &Path) -> Result<Self> {
        let values = if path.exists() {
            let contents =
                fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
            if contents.trim().is_empty() {
                BTreeMap::new()
            } else {
                let parsed: Map<String, Value> = serde_json::from_str(&contents)
                    .with_context(|| format!("parsing {}", path.display()))?;
                parsed.into_iter().collect()
            }
        } else {
            BTreeMap::new()
        };

        Ok(Self {
            path: path.to_path_buf(),
            values,
            dirty: false,
        })
    }

    /// Returns the resolved default settings file path for the current OS.
    pub fn default_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("could not determine home directory")?;
        Ok(home.join(".config").join("amp").join("settings.json"))
    }

    /// Gets the current value for a key, falling back to the known default.
    pub fn get(&self, key: &str) -> Value {
        if let Some(val) = self.values.get(key) {
            val.clone()
        } else if let Some(def) = settings::get_setting_def(key) {
            def.default.clone()
        } else {
            Value::Null
        }
    }

    /// Gets the raw value for a key (None if not explicitly set).
    pub fn get_raw(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    /// Sets a value for a key.
    pub fn set(&mut self, key: &str, value: Value) {
        self.values.insert(key.to_string(), value);
        self.dirty = true;
    }

    /// Removes a key (resets to default).
    pub fn remove(&mut self, key: &str) {
        if self.values.remove(key).is_some() {
            self.dirty = true;
        }
    }

    /// Returns whether the config has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Saves the config to disk as formatted JSON.
    pub fn save(&mut self) -> Result<()> {
        let map: Map<String, Value> = self
            .values
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let json =
            serde_json::to_string_pretty(&Value::Object(map)).context("serializing settings")?;

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating directory {}", parent.display()))?;
        }

        fs::write(&self.path, json + "\n")
            .with_context(|| format!("writing {}", self.path.display()))?;

        self.dirty = false;
        Ok(())
    }

    /// Returns all keys that are not known settings (for the Advanced section).
    pub fn unknown_keys(&self) -> Vec<String> {
        self.values
            .keys()
            .filter(|k| settings::section_for_key(k).is_none())
            .cloned()
            .collect()
    }

    /// Validates that a value matches the expected type for a known setting.
    pub fn validate_value(key: &str, value: &Value) -> Result<()> {
        let Some(def) = settings::get_setting_def(key) else {
            return Ok(());
        };

        let type_ok = match def.setting_type {
            SettingType::Boolean => value.is_boolean(),
            SettingType::String | SettingType::StringEnum => value.is_string(),
            SettingType::Number => value.is_number(),
            SettingType::ArrayString => {
                value.is_array()
                    && value
                        .as_array()
                        .map(|a| a.iter().all(|v| v.is_string()))
                        .unwrap_or(false)
            }
            SettingType::ArrayObject => {
                value.is_array()
                    && value
                        .as_array()
                        .map(|a| a.iter().all(|v| v.is_object()))
                        .unwrap_or(false)
            }
            SettingType::Object => value.is_object(),
        };

        anyhow::ensure!(
            type_ok,
            "expected {} for key '{}'",
            match def.setting_type {
                SettingType::Boolean => "boolean",
                SettingType::String | SettingType::StringEnum => "string",
                SettingType::Number => "number",
                SettingType::ArrayString => "array of strings",
                SettingType::ArrayObject => "array of objects",
                SettingType::Object => "object",
            },
            key
        );

        if def.setting_type == SettingType::StringEnum {
            if let (Some(options), Some(s)) = (def.enum_options, value.as_str()) {
                if !options.contains(&s) {
                    anyhow::bail!(
                        "invalid value '{}' for '{}', expected one of: {}",
                        s,
                        key,
                        options.join(", ")
                    );
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn sample_json() -> &'static str {
        r#"{
    "amp.showCosts": true,
    "amp.notifications.enabled": false,
    "amp.tools.stopTimeout": 600,
    "amp.experimental.modes": ["bombadil"]
}"#
    }

    #[test]
    fn test_load_existing_file() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "{}", sample_json()).unwrap();

        let config = Config::load(f.path()).unwrap();
        assert_eq!(config.get("amp.showCosts"), Value::Bool(true));
        assert_eq!(config.get("amp.notifications.enabled"), Value::Bool(false));
        assert_eq!(
            config.get("amp.tools.stopTimeout"),
            Value::Number(600.into())
        );
        assert!(!config.is_dirty());
    }

    #[test]
    fn test_load_missing_file() {
        let config = Config::load(Path::new("/tmp/nonexistent-volt-test.json")).unwrap();
        // Missing keys fall back to defaults
        assert_eq!(config.get("amp.showCosts"), Value::Bool(true));
        assert_eq!(
            config.get("amp.tools.stopTimeout"),
            Value::Number(300.into())
        );
    }

    #[test]
    fn test_load_invalid_json() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "not json").unwrap();
        assert!(Config::load(f.path()).is_err());
    }

    #[test]
    fn test_set_and_dirty() {
        let config_path = Path::new("/tmp/nonexistent-volt-test.json");
        let mut config = Config::load(config_path).unwrap();
        assert!(!config.is_dirty());

        config.set("amp.showCosts", Value::Bool(false));
        assert!(config.is_dirty());
        assert_eq!(config.get("amp.showCosts"), Value::Bool(false));
    }

    #[test]
    fn test_remove_resets_to_default() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, r#"{{"amp.showCosts": false}}"#).unwrap();

        let mut config = Config::load(f.path()).unwrap();
        assert_eq!(config.get("amp.showCosts"), Value::Bool(false));

        config.remove("amp.showCosts");
        assert!(config.is_dirty());
        // Falls back to default
        assert_eq!(config.get("amp.showCosts"), Value::Bool(true));
    }

    #[test]
    fn test_unknown_keys() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "{}", sample_json()).unwrap();

        let config = Config::load(f.path()).unwrap();
        let unknown = config.unknown_keys();
        assert!(unknown.contains(&"amp.experimental.modes".to_string()));
        assert!(!unknown.contains(&"amp.showCosts".to_string()));
    }

    #[test]
    fn test_save_roundtrip() {
        let tmpfile = NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_path_buf();
        // Keep tmpfile alive so the file isn't deleted
        let _keep = tmpfile;

        let mut config = Config::load(&path).unwrap();
        config.set("amp.showCosts", Value::Bool(false));
        config.set("amp.tools.stopTimeout", Value::Number(120.into()));
        config.save().unwrap();
        assert!(!config.is_dirty());

        let reloaded = Config::load(&path).unwrap();
        assert_eq!(reloaded.get("amp.showCosts"), Value::Bool(false));
        assert_eq!(
            reloaded.get("amp.tools.stopTimeout"),
            Value::Number(120.into())
        );
    }

    #[test]
    fn test_validate_boolean() {
        assert!(Config::validate_value("amp.showCosts", &Value::Bool(true)).is_ok());
        assert!(Config::validate_value("amp.showCosts", &Value::String("yes".into())).is_err());
    }

    #[test]
    fn test_validate_number() {
        assert!(
            Config::validate_value("amp.tools.stopTimeout", &Value::Number(100.into())).is_ok()
        );
        assert!(Config::validate_value("amp.tools.stopTimeout", &Value::Bool(true)).is_err());
    }

    #[test]
    fn test_validate_enum() {
        assert!(Config::validate_value("amp.updates.mode", &Value::String("auto".into())).is_ok());
        assert!(
            Config::validate_value("amp.updates.mode", &Value::String("invalid".into())).is_err()
        );
    }

    #[test]
    fn test_validate_array_string() {
        let val = Value::Array(vec![Value::String("*.rs".into())]);
        assert!(Config::validate_value("amp.fuzzy.alwaysIncludePaths", &val).is_ok());

        let bad = Value::Array(vec![Value::Number(42.into())]);
        assert!(Config::validate_value("amp.fuzzy.alwaysIncludePaths", &bad).is_err());
    }

    #[test]
    fn test_validate_unknown_key_always_ok() {
        assert!(Config::validate_value("some.unknown", &Value::Bool(true)).is_ok());
    }

    #[test]
    fn test_default_path() {
        let path = Config::default_path().unwrap();
        assert!(path.ends_with(".config/amp/settings.json"));
    }

    #[test]
    fn test_preserve_unknown_keys_on_save() {
        let tmpfile = NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_path_buf();
        let _keep = tmpfile;

        let mut config = Config::load(&path).unwrap();
        config.set("amp.showCosts", Value::Bool(false));
        config.set(
            "amp.experimental.modes",
            Value::Array(vec![Value::String("test".into())]),
        );
        config.save().unwrap();

        let reloaded = Config::load(&path).unwrap();
        assert_eq!(
            reloaded.get("amp.experimental.modes"),
            Value::Array(vec![Value::String("test".into())])
        );
        assert_eq!(reloaded.get("amp.showCosts"), Value::Bool(false));
    }
}
