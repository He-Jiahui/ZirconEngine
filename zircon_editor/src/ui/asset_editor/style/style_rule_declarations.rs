use std::collections::BTreeMap;

use toml::{map::Map, Value};
use zircon_runtime::ui::template::UiStyleDeclarationBlock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiStyleRuleDeclarationEntry {
    pub path: String,
    pub literal: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiStyleRuleDeclarationPath {
    target: UiStyleRuleDeclarationTarget,
    segments: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UiStyleRuleDeclarationTarget {
    SelfValues,
    Slot,
}

impl UiStyleRuleDeclarationPath {
    pub(crate) fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        let mut segments = trimmed.split('.').map(str::trim);
        let target = match segments.next()? {
            "self" => UiStyleRuleDeclarationTarget::SelfValues,
            "slot" => UiStyleRuleDeclarationTarget::Slot,
            _ => return None,
        };

        let segments = segments.map(str::to_string).collect::<Vec<_>>();
        if segments.is_empty()
            || segments
                .iter()
                .any(|segment| segment.is_empty() || segment.chars().any(char::is_whitespace))
        {
            return None;
        }

        Some(Self { target, segments })
    }

    pub(crate) fn display(&self) -> String {
        let mut path = match self.target {
            UiStyleRuleDeclarationTarget::SelfValues => "self".to_string(),
            UiStyleRuleDeclarationTarget::Slot => "slot".to_string(),
        };
        for segment in &self.segments {
            path.push('.');
            path.push_str(segment);
        }
        path
    }
}

pub(crate) fn declaration_entries(
    block: &UiStyleDeclarationBlock,
) -> Vec<UiStyleRuleDeclarationEntry> {
    let mut entries = Vec::new();
    collect_map_entries(&mut entries, "self", &block.self_values);
    collect_map_entries(&mut entries, "slot", &block.slot);
    entries
}

pub(crate) fn parse_declaration_literal(value_literal: &str) -> Option<Value> {
    let trimmed = value_literal.trim();
    if trimmed.is_empty() {
        return None;
    }

    let wrapped = format!("value = {trimmed}");
    toml::from_str::<toml::Table>(&wrapped)
        .ok()
        .and_then(|table| table.get("value").cloned())
        .or_else(|| Some(Value::String(trimmed.to_string())))
}

pub(crate) fn set_declaration(
    block: &mut UiStyleDeclarationBlock,
    path: &UiStyleRuleDeclarationPath,
    value: Value,
) {
    match path.target {
        UiStyleRuleDeclarationTarget::SelfValues => {
            set_in_value_map(&mut block.self_values, &path.segments, value);
        }
        UiStyleRuleDeclarationTarget::Slot => {
            set_in_value_map(&mut block.slot, &path.segments, value);
        }
    }
}

pub(crate) fn remove_declaration(
    block: &mut UiStyleDeclarationBlock,
    path: &UiStyleRuleDeclarationPath,
) -> bool {
    match path.target {
        UiStyleRuleDeclarationTarget::SelfValues => {
            remove_from_value_map(&mut block.self_values, &path.segments)
        }
        UiStyleRuleDeclarationTarget::Slot => {
            remove_from_value_map(&mut block.slot, &path.segments)
        }
    }
}

fn collect_map_entries(
    output: &mut Vec<UiStyleRuleDeclarationEntry>,
    prefix: &str,
    values: &BTreeMap<String, Value>,
) {
    for (key, value) in values {
        collect_value_entries(output, &format!("{prefix}.{key}"), value);
    }
}

fn collect_value_entries(output: &mut Vec<UiStyleRuleDeclarationEntry>, path: &str, value: &Value) {
    match value {
        Value::Table(table) if !table.is_empty() => {
            for (key, child) in table {
                collect_value_entries(output, &format!("{path}.{key}"), child);
            }
        }
        _ => output.push(UiStyleRuleDeclarationEntry {
            path: path.to_string(),
            literal: value.to_string(),
        }),
    }
}

fn set_in_value_map(values: &mut BTreeMap<String, Value>, segments: &[String], value: Value) {
    let Some((first, rest)) = segments.split_first() else {
        return;
    };
    if rest.is_empty() {
        let _ = values.insert(first.clone(), value);
        return;
    }

    let entry = values
        .entry(first.clone())
        .or_insert_with(|| Value::Table(Map::new()));
    if !matches!(entry, Value::Table(_)) {
        *entry = Value::Table(Map::new());
    }
    let Value::Table(table) = entry else {
        unreachable!("value should be replaced with a table before recursion");
    };
    set_in_table(table, rest, value);
}

fn set_in_table(values: &mut Map<String, Value>, segments: &[String], value: Value) {
    let Some((first, rest)) = segments.split_first() else {
        return;
    };
    if rest.is_empty() {
        let _ = values.insert(first.clone(), value);
        return;
    }

    let entry = values
        .entry(first.clone())
        .or_insert_with(|| Value::Table(Map::new()));
    if !matches!(entry, Value::Table(_)) {
        *entry = Value::Table(Map::new());
    }
    let Value::Table(table) = entry else {
        unreachable!("value should be replaced with a table before recursion");
    };
    set_in_table(table, rest, value);
}

fn remove_from_value_map(values: &mut BTreeMap<String, Value>, segments: &[String]) -> bool {
    let Some((first, rest)) = segments.split_first() else {
        return false;
    };
    if rest.is_empty() {
        return values.remove(first).is_some();
    }

    let Some(entry) = values.get_mut(first) else {
        return false;
    };
    let Value::Table(table) = entry else {
        return false;
    };
    let removed = remove_from_table(table, rest);
    if removed && table.is_empty() {
        let _ = values.remove(first);
    }
    removed
}

fn remove_from_table(values: &mut Map<String, Value>, segments: &[String]) -> bool {
    let Some((first, rest)) = segments.split_first() else {
        return false;
    };
    if rest.is_empty() {
        return values.remove(first).is_some();
    }

    let Some(entry) = values.get_mut(first) else {
        return false;
    };
    let Value::Table(table) = entry else {
        return false;
    };
    let removed = remove_from_table(table, rest);
    if removed && table.is_empty() {
        let _ = values.remove(first);
    }
    removed
}
