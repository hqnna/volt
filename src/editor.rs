//! External editor support for editing JSON values via `$EDITOR`.

use std::env;
use std::fs;
use std::process::Command;

use anyhow::{Context, Result};
use serde_json::Value;

/// Opens a JSON value in the user's `$EDITOR`, waits for save & quit,
/// then reads back and parses the result.
pub fn edit_value_in_editor(value: &Value) -> Result<Value> {
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    let json = serde_json::to_string_pretty(value).context("serializing value for editor")?;

    let tmp = tempfile::Builder::new()
        .suffix(".json")
        .tempfile()
        .context("creating temp file")?;

    fs::write(tmp.path(), &json).context("writing temp file")?;

    let status = Command::new(&editor)
        .arg(tmp.path())
        .status()
        .with_context(|| format!("launching editor '{editor}'"))?;

    if !status.success() {
        anyhow::bail!("editor exited with {status}");
    }

    let edited = fs::read_to_string(tmp.path()).context("reading edited file")?;
    let parsed: Value = serde_json::from_str(&edited).context("parsing edited JSON")?;

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_value_with_true_editor() {
        // Use `true` as editor (no-op, exits 0, file unchanged)
        env::set_var("EDITOR", "true");
        let original = Value::Object(serde_json::Map::new());
        let result = edit_value_in_editor(&original).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_edit_value_with_failing_editor() {
        env::set_var("EDITOR", "false");
        let original = Value::Object(serde_json::Map::new());
        let result = edit_value_in_editor(&original);
        assert!(result.is_err());
    }
}
