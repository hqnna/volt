//! UI rendering for the Volt TUI.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table};
use ratatui::Frame;
use serde_json::Value;

use crate::app::{App, CustomKeyType, Focus, InputMode, SettingEntry};
use crate::settings::{Section, SettingType};

/// Sidebar width in columns.
const SIDEBAR_WIDTH: u16 = 18;

/// Renders the full application UI.
pub fn render(frame: &mut Frame, app: &App) {
    let status_rows = if app.status_message.is_some() { 2 } else { 1 };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(status_rows)])
        .split(frame.area());

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(1)])
        .split(rows[0]);

    render_sidebar(frame, app, columns[0]);
    render_settings_panel(frame, app, columns[1]);
    render_bottom_bar(frame, app, rows[1]);

    if app.is_editing() {
        render_edit_overlay(frame, app);
    }
}

/// Renders the sidebar with section tabs.
fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let title = if app.config.is_dirty() {
        " Volt [modified] "
    } else {
        " Volt "
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.focus == Focus::Sidebar {
            Color::Cyan
        } else {
            Color::DarkGray
        }));

    let items: Vec<ListItem> = Section::ALL
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let style = if i == app.selected_section {
                if app.focus == Focus::Sidebar {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                }
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!(" {} ", section.label())).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

/// Renders the settings panel for the current section.
fn render_settings_panel(frame: &mut Frame, app: &App, area: Rect) {
    let section = app.current_section();
    let block = Block::default()
        .title(format!(" {} ", section.label()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.focus == Focus::Settings {
            Color::Cyan
        } else {
            Color::DarkGray
        }));

    if section.is_single_key() {
        render_single_key_panel(frame, app, area, block);
        return;
    }

    let entries = app.current_settings();

    if entries.is_empty() {
        let help = if section == Section::Advanced {
            "No custom keys. Press 'a' to add one."
        } else {
            "No settings in this section."
        };
        let p = Paragraph::new(help)
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(p, area);
        return;
    }

    let selected_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let rows: Vec<Row> = entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = app.focus == Focus::Settings && i == app.selected_setting;
            let base = if is_selected {
                selected_style
            } else {
                Style::default()
            };
            let value_style = if is_selected {
                base
            } else {
                Style::default().fg(Color::Yellow)
            };

            let (key, value_display, modified) = match entry {
                SettingEntry::Known(def) => {
                    let value = app.config.get(def.key);
                    let display = format_value(def.setting_type, &value);
                    let modified = app.config.get_raw(def.key).is_some();
                    (def.key.to_string(), display, modified)
                }
                SettingEntry::Unknown(key) => {
                    let value = app.config.get(key);
                    let display = format_json_compact(&value);
                    (key.clone(), display, true)
                }
            };

            let key_style = if modified {
                base.add_modifier(Modifier::BOLD)
            } else {
                base
            };

            Row::new(vec![
                Line::from(Span::styled(format!(" {key}"), key_style)),
                Line::from(Span::styled(value_display, value_style)),
            ])
            .style(base)
        })
        .collect();

    let table = Table::new(rows, [Constraint::Fill(1), Constraint::Min(16)])
        .block(block)
        .row_highlight_style(selected_style)
        .column_spacing(2);

    frame.render_widget(table, area);
}

/// Renders a single-key section where the right panel shows array items directly.
fn render_single_key_panel(frame: &mut Frame, app: &App, area: Rect, block: Block) {
    let entries = app.current_settings();
    let def = match entries.first() {
        Some(SettingEntry::Known(def)) => def,
        _ => {
            let p = Paragraph::new("No settings in this section.")
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            frame.render_widget(p, area);
            return;
        }
    };

    let value = app.config.get(def.key);
    let items = value.as_array().cloned().unwrap_or_default();

    if items.is_empty() {
        let p = Paragraph::new(" Empty. Press 'a' to add an item, 'e' to open in $EDITOR.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(p, area);
        return;
    }

    let selected_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    // Collect all unique keys across objects to build columns.
    let columns = collect_object_columns(&items);

    if columns.is_empty() {
        // Non-object items: fall back to a simple list.
        let list_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = app.focus == Focus::Settings && i == app.selected_setting;
                let style = if is_selected {
                    selected_style
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(format!(" {}", format_json_compact(item))).style(style)
            })
            .collect();
        let list = List::new(list_items).block(block);
        frame.render_widget(list, area);
        return;
    }

    // Build header row.
    let header = Row::new(
        columns
            .iter()
            .map(|col| {
                Line::from(Span::styled(
                    col.as_str(),
                    Style::default().fg(Color::DarkGray),
                ))
            })
            .collect::<Vec<_>>(),
    );

    // Build data rows.
    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = app.focus == Focus::Settings && i == app.selected_setting;
            let base = if is_selected {
                selected_style
            } else {
                Style::default()
            };
            let value_style = if is_selected {
                base
            } else {
                Style::default().fg(Color::Yellow)
            };
            let cells: Vec<Line> = columns
                .iter()
                .map(|col| {
                    let text = item
                        .get(col)
                        .map(|v| match v {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .unwrap_or_default();
                    Line::from(Span::styled(text, value_style))
                })
                .collect();
            Row::new(cells).style(base)
        })
        .collect();

    let widths: Vec<Constraint> = columns.iter().map(|_| Constraint::Fill(1)).collect();
    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .column_spacing(2);

    frame.render_widget(table, area);
}

/// Collects unique object field names from an array of values.
/// Columns are sorted so identifying fields (tool, name, pattern) appear first.
fn collect_object_columns(items: &[Value]) -> Vec<String> {
    let mut columns: Vec<String> = Vec::new();
    for item in items {
        if let Some(obj) = item.as_object() {
            for key in obj.keys() {
                if !columns.contains(key) {
                    columns.push(key.clone());
                }
            }
        } else {
            return Vec::new();
        }
    }
    columns.sort_by_key(|k| column_priority(k));
    columns
}

/// Returns a sort priority for a column name. Lower values appear first.
fn column_priority(name: &str) -> u8 {
    match name {
        "tool" | "name" | "pattern" | "key" => 0,
        "action" | "decision" | "type" => 1,
        _ => 2,
    }
}

/// Formats a value for display based on its type.
fn format_value(setting_type: SettingType, value: &Value) -> String {
    match setting_type {
        SettingType::Boolean => {
            if value.as_bool().unwrap_or(false) {
                "[✓]".to_string()
            } else {
                "[✗]".to_string()
            }
        }
        SettingType::String | SettingType::StringEnum => {
            let s = value.as_str().unwrap_or("");
            if s.is_empty() {
                "(empty)".to_string()
            } else {
                format!("\"{}\"", s)
            }
        }
        SettingType::Number => match value.as_f64() {
            Some(n) => {
                if n.fract() == 0.0 {
                    format!("{}", n as i64)
                } else {
                    format!("{}", n)
                }
            }
            None => "0".to_string(),
        },
        SettingType::ArrayString => {
            let arr = value.as_array();
            match arr {
                Some(a) if a.is_empty() => "[]".to_string(),
                Some(a) => {
                    let items: Vec<&str> = a.iter().filter_map(|v| v.as_str()).collect();
                    format!("[{}]", items.join(", "))
                }
                None => "[]".to_string(),
            }
        }
        SettingType::ArrayObject => {
            let arr = value.as_array();
            match arr {
                Some(a) if a.is_empty() => "[]".to_string(),
                Some(a) => format!("[{} items]", a.len()),
                None => "[]".to_string(),
            }
        }
        SettingType::Object => {
            let obj = value.as_object();
            match obj {
                Some(o) if o.is_empty() => "{}".to_string(),
                Some(o) => format!("{{{} keys}}", o.len()),
                None => "{}".to_string(),
            }
        }
    }
}

/// Formats a JSON value compactly for display.
fn format_json_compact(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s),
        Value::Bool(b) => {
            if *b {
                "[✓]".to_string()
            } else {
                "[✗]".to_string()
            }
        }
        Value::Number(n) => n.to_string(),
        Value::Array(a) if a.is_empty() => "[]".to_string(),
        Value::Array(a) => {
            let items: Vec<String> = a.iter().map(format_json_compact).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(o) if o.is_empty() => "{}".to_string(),
        Value::Object(o) => format!("{{{} keys}}", o.len()),
        Value::Null => "null".to_string(),
    }
}

/// Renders the bottom bar area (help line + optional status message).
fn render_bottom_bar(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref msg) = app.status_message {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(area);

        render_help_line(frame, app, rows[0]);

        let bar =
            Paragraph::new(msg.as_str()).style(Style::default().fg(Color::Black).bg(Color::Yellow));
        frame.render_widget(bar, rows[1]);
    } else {
        render_help_line(frame, app, area);
    }
}

/// Renders the help/description line.
fn render_help_line(frame: &mut Frame, app: &App, area: Rect) {
    let text = if app.focus == Focus::Settings {
        let section = app.current_section();
        if section == Section::Advanced {
            " Enter: edit | a: add key | r: remove | e: $EDITOR | Tab: sidebar".to_string()
        } else if section.is_single_key() {
            " Enter: edit item | a: add | d: delete | e: $EDITOR | r: reset | Tab: sidebar"
                .to_string()
        } else {
            let entries = app.current_settings();
            let is_array = entries.get(app.selected_setting).is_some_and(|e| {
                matches!(
                    e,
                    SettingEntry::Known(d)
                        if matches!(d.setting_type, SettingType::ArrayString | SettingType::ArrayObject)
                )
            });
            if is_array {
                " Enter: toggle/edit | a: add | d: delete | r: reset | e: $EDITOR | Tab: sidebar"
                    .to_string()
            } else {
                " Enter: toggle/edit | r: reset | e: $EDITOR | Tab: sidebar".to_string()
            }
        }
    } else {
        format!(
            " ↑↓: navigate | Enter/Tab: settings | Ctrl+S: save | q: quit | {}",
            app.config.path().display()
        )
    };

    let bar = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(bar, area);
}

/// Renders the appropriate edit overlay based on input mode.
fn render_edit_overlay(frame: &mut Frame, app: &App) {
    match app.input_mode {
        InputMode::SelectingType => render_type_select_overlay(frame, app),
        InputMode::Normal => {}
        _ => render_text_input_overlay(frame, app),
    }
}

/// Renders a text input overlay for inline editing, key name entry, or custom value entry.
fn render_text_input_overlay(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let width = 50.min(area.width.saturating_sub(4));
    let height = 3;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup_area);

    let title = match app.input_mode {
        InputMode::EnteringKeyName => " Enter Key Name (Enter to confirm, Esc to cancel) ",
        InputMode::EnteringCustomValue => " Enter Value (Enter to save, Esc to cancel) ",
        _ => " Edit Value (Enter to save, Esc to cancel) ",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let input = Paragraph::new(app.edit_buffer.as_str())
        .style(Style::default().fg(Color::White))
        .block(block);

    frame.render_widget(input, popup_area);
}

/// Renders the type selection overlay for choosing a custom key value type.
fn render_type_select_overlay(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let item_count = CustomKeyType::ALL.len() as u16;
    let width = 40.min(area.width.saturating_sub(4));
    let height = (item_count + 2).min(area.height.saturating_sub(2)); // +2 for border
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Select Type (Enter to confirm, Esc to cancel) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let selected_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    let items: Vec<ListItem> = CustomKeyType::ALL
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let style = if i == app.selected_type {
                selected_style
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!("  {}", t.label())).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, popup_area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_value_boolean() {
        assert_eq!(
            format_value(SettingType::Boolean, &Value::Bool(true)),
            "[✓]"
        );
        assert_eq!(
            format_value(SettingType::Boolean, &Value::Bool(false)),
            "[✗]"
        );
    }

    #[test]
    fn test_format_value_string() {
        assert_eq!(
            format_value(SettingType::String, &Value::String("hello".into())),
            "\"hello\""
        );
        assert_eq!(
            format_value(SettingType::String, &Value::String(String::new())),
            "(empty)"
        );
    }

    #[test]
    fn test_format_value_number() {
        assert_eq!(
            format_value(SettingType::Number, &Value::Number(300.into())),
            "300"
        );
    }

    #[test]
    fn test_format_value_array_string() {
        assert_eq!(
            format_value(SettingType::ArrayString, &Value::Array(vec![])),
            "[]"
        );
        assert_eq!(
            format_value(
                SettingType::ArrayString,
                &Value::Array(vec![Value::String("a".into()), Value::String("b".into())])
            ),
            "[a, b]"
        );
    }

    #[test]
    fn test_format_value_array_object() {
        assert_eq!(
            format_value(SettingType::ArrayObject, &Value::Array(vec![])),
            "[]"
        );
        assert_eq!(
            format_value(
                SettingType::ArrayObject,
                &Value::Array(vec![Value::Object(serde_json::Map::new())])
            ),
            "[1 items]"
        );
    }

    #[test]
    fn test_format_value_object() {
        assert_eq!(
            format_value(SettingType::Object, &Value::Object(serde_json::Map::new())),
            "{}"
        );
    }

    #[test]
    fn test_collect_object_columns() {
        let mut obj1 = serde_json::Map::new();
        obj1.insert("tool".into(), Value::String("Bash".into()));
        obj1.insert("action".into(), Value::String("allow".into()));
        let mut obj2 = serde_json::Map::new();
        obj2.insert("tool".into(), Value::String("Read".into()));
        obj2.insert("action".into(), Value::String("ask".into()));
        let items = vec![Value::Object(obj1), Value::Object(obj2)];
        let cols = collect_object_columns(&items);
        assert_eq!(cols, vec!["tool", "action"]);
    }

    #[test]
    fn test_collect_object_columns_non_objects() {
        let items = vec![Value::String("a".into()), Value::String("b".into())];
        assert!(collect_object_columns(&items).is_empty());
    }

    #[test]
    fn test_collect_object_columns_mixed() {
        let mut obj = serde_json::Map::new();
        obj.insert("key".into(), Value::String("val".into()));
        let items = vec![Value::Object(obj), Value::String("not an object".into())];
        assert!(collect_object_columns(&items).is_empty());
    }

    #[test]
    fn test_format_json_compact() {
        assert_eq!(format_json_compact(&Value::Null), "null");
        assert_eq!(format_json_compact(&Value::Bool(true)), "[✓]");
        assert_eq!(
            format_json_compact(&Value::String("test".into())),
            "\"test\""
        );
    }

    #[test]
    fn test_format_json_compact_array() {
        assert_eq!(format_json_compact(&Value::Array(vec![])), "[]");
        assert_eq!(
            format_json_compact(&Value::Array(vec![
                Value::String("a".into()),
                Value::String("b".into())
            ])),
            "[\"a\", \"b\"]"
        );
    }
}
