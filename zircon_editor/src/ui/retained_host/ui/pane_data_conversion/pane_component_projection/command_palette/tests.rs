use std::collections::BTreeMap;

use super::{projected_command_palette_options, projected_command_palette_structured_options};
use toml::Value;

#[test]
fn filtered_commands_preserve_order_and_project_state() {
    let attributes = command_attributes([
        ("selected_command_id", Value::String("build.run".into())),
        ("focused_index", Value::Integer(1)),
        ("query", Value::String("build".into())),
        (
            "recent_commands",
            Value::Array(vec![Value::String("project.open".into())]),
        ),
        (
            "filtered_commands",
            Value::Array(vec![
                Value::String("project.open".into()),
                command_id_table([("id", Value::String("build.run".into()))]),
                Value::String("missing.command|label=Missing Command".into()),
            ]),
        ),
        (
            "commands",
            Value::Array(vec![
                command_entry([
                    ("id", Value::String("build.run".into())),
                    ("label", Value::String("Run Build".into())),
                    ("shortcut", Value::String("Ctrl+B".into())),
                ]),
                Value::String("project.open|label=Open Project|shortcut=Ctrl+O".into()),
            ]),
        ),
    ]);

    assert_eq!(
        projected_command_palette_options("command-palette", &attributes),
        Some(vec![
            "Open Project".to_string(),
            "Run Build".to_string(),
            "missing.command".to_string()
        ])
    );

    let rows = projected_command_palette_structured_options("command-palette", &attributes)
        .expect("command rows should project");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].id.as_str(), "project.open");
    assert_eq!(rows[0].label.as_str(), "Open Project");
    assert_eq!(rows[0].description.as_str(), "Ctrl+O");
    assert!(!rows[0].disabled);
    assert!(rows[0].special);
    assert!(rows[0].matched);
    assert_eq!(rows[1].id.as_str(), "build.run");
    assert_eq!(rows[1].label.as_str(), "Run Build");
    assert_eq!(rows[1].description.as_str(), "Ctrl+B");
    assert!(rows[1].selected);
    assert!(rows[1].focused);
    assert!(rows[1].matched);
    assert_eq!(rows[2].id.as_str(), "missing.command");
    assert_eq!(rows[2].label.as_str(), "missing.command");
}

#[test]
fn table_entries_use_enabled_false_as_disabled() {
    let attributes = command_attributes([(
        "commands",
        Value::Array(vec![command_entry([
            ("commandId", Value::String("editor.save".into())),
            ("value_text", Value::String("Save Scene".into())),
            ("keybinding", Value::String("Ctrl+S".into())),
            ("enabled", Value::Boolean(false)),
        ])]),
    )]);

    let rows = projected_command_palette_structured_options("command-palette", &attributes)
        .expect("table entry should project");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id.as_str(), "editor.save");
    assert_eq!(rows[0].label.as_str(), "Save Scene");
    assert_eq!(rows[0].description.as_str(), "Ctrl+S");
    assert!(rows[0].disabled);
}

#[test]
fn filtered_commands_use_the_first_duplicate_and_match_ascii_case_insensitively() {
    let attributes = command_attributes([
        ("query", Value::String("BUILD".into())),
        (
            "filtered_commands",
            Value::Array(vec![Value::String("build.run".into())]),
        ),
        (
            "commands",
            Value::Array(vec![
                command_entry([
                    ("id", Value::String("build.run".into())),
                    ("label", Value::String("Run Build".into())),
                ]),
                command_entry([
                    ("id", Value::String("build.run".into())),
                    ("label", Value::String("Duplicate".into())),
                ]),
            ]),
        ),
    ]);

    let rows = projected_command_palette_structured_options("command-palette", &attributes)
        .expect("command rows should project");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].label.as_str(), "Run Build");
    assert!(rows[0].matched);
}

#[test]
fn non_command_palette_roles_do_not_claim_options() {
    let attributes = command_attributes([(
        "commands",
        Value::Array(vec![Value::String("editor.save|label=Save Scene".into())]),
    )]);

    assert_eq!(
        projected_command_palette_options("notification-center", &attributes),
        None
    );
    assert_eq!(
        projected_command_palette_structured_options("notification-center", &attributes),
        None
    );
}

fn command_attributes(
    attributes: impl IntoIterator<Item = (&'static str, Value)>,
) -> BTreeMap<String, Value> {
    attributes
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect()
}

fn command_entry(values: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Table(command_table(values))
}

fn command_id_table(values: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Table(command_table(values))
}

fn command_table(
    values: impl IntoIterator<Item = (&'static str, Value)>,
) -> toml::map::Map<String, Value> {
    let mut table = toml::map::Map::new();
    for (name, value) in values {
        table.insert(name.to_owned(), value);
    }
    table
}
