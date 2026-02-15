//! UI rendering for the Volt TUI.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;
use serde_json::Value;

use crate::app::{App, Focus, SettingEntry};
use crate::settings::{Section, SettingType};

/// Sidebar width in columns.
const SIDEBAR_WIDTH: u16 = 18;

/// Renders the full application UI.
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(1)])
        .split(frame.area());

    render_sidebar(frame, app, chunks[0]);
    render_settings_panel(frame, app, chunks[1]);

    render_help_bar(frame, app);

    if let Some(ref msg) = app.status_message {
        render_status_bar(frame, msg);
    }

    if app.editing {
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

    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = app.focus == Focus::Settings && i == app.selected_setting;
            render_setting_item(app, entry, is_selected)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

/// Renders a single setting entry as a ListItem.
fn render_setting_item(app: &App, entry: &SettingEntry, selected: bool) -> ListItem<'static> {
    let style = if selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let line = match entry {
        SettingEntry::Known(def) => {
            let value = app.config.get(def.key);
            let value_display = format_value(def.setting_type, &value);
            let is_modified = app.config.get_raw(def.key).is_some();

            let mut spans = vec![Span::styled(
                format!("  {}  ", def.key),
                if is_modified {
                    style.add_modifier(Modifier::BOLD)
                } else {
                    style
                },
            )];

            spans.push(Span::styled(value_display, style.fg(Color::Yellow)));

            Line::from(spans)
        }
        SettingEntry::Unknown(key) => {
            let value = app.config.get(key);
            let display = format_json_compact(&value);
            Line::from(vec![
                Span::styled(format!("  {}  ", key), style),
                Span::styled(display, style.fg(Color::Yellow)),
            ])
        }
    };

    ListItem::new(line).style(style)
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
        SettingType::ArrayString | SettingType::ArrayObject => {
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
        Value::Array(a) => format!("[{} items]", a.len()),
        Value::Object(o) if o.is_empty() => "{}".to_string(),
        Value::Object(o) => format!("{{{} keys}}", o.len()),
        Value::Null => "null".to_string(),
    }
}

/// Renders a help bar showing the description of the currently selected setting.
fn render_help_bar(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let bar_area = Rect::new(0, area.height.saturating_sub(2), area.width, 1);

    let text = if app.focus == Focus::Settings {
        let entries = app.current_settings();
        entries
            .get(app.selected_setting)
            .map(|entry| match entry {
                SettingEntry::Known(def) => {
                    format!(
                        " {} — {} ({})",
                        def.key,
                        def.description,
                        app.config.path().display()
                    )
                }
                SettingEntry::Unknown(key) => format!(" {} (custom key)", key),
            })
            .unwrap_or_default()
    } else {
        format!(
            " q: quit | Tab: switch panel | ↑↓: navigate | Ctrl+S: save | {}",
            app.config.path().display()
        )
    };

    let bar = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(bar, bar_area);
}

/// Renders a status bar at the bottom of the screen.
fn render_status_bar(frame: &mut Frame, message: &str) {
    let area = frame.area();
    let bar_area = Rect::new(0, area.height.saturating_sub(1), area.width, 1);
    let bar = Paragraph::new(message).style(Style::default().fg(Color::Black).bg(Color::Yellow));
    frame.render_widget(bar, bar_area);
}

/// Renders an inline edit overlay for string/number editing.
fn render_edit_overlay(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let width = 50.min(area.width.saturating_sub(4));
    let height = 3;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Edit Value (Enter to save, Esc to cancel) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let input = Paragraph::new(app.edit_buffer.as_str())
        .style(Style::default().fg(Color::White))
        .block(block);

    frame.render_widget(input, popup_area);
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
    fn test_format_value_array() {
        assert_eq!(
            format_value(SettingType::ArrayString, &Value::Array(vec![])),
            "[]"
        );
        assert_eq!(
            format_value(
                SettingType::ArrayString,
                &Value::Array(vec![Value::String("a".into()), Value::String("b".into())])
            ),
            "[2 items]"
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
    fn test_format_json_compact() {
        assert_eq!(format_json_compact(&Value::Null), "null");
        assert_eq!(format_json_compact(&Value::Bool(true)), "[✓]");
        assert_eq!(
            format_json_compact(&Value::String("test".into())),
            "\"test\""
        );
    }
}
