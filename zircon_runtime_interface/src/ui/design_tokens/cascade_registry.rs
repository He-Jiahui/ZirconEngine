use std::collections::BTreeMap;

use toml::Value;

use crate::ui::style::UiRgbaColor;

use super::EditorDesignTokens;

pub(super) fn cascade_token_values(tokens: &EditorDesignTokens) -> BTreeMap<String, Value> {
    let mut values = canonical_cascade_token_values(tokens);
    values.extend(custom_property_aliases(&values));

    for (legacy_name, canonical_name) in legacy_density_token_aliases() {
        values.insert(
            legacy_name.to_string(),
            Value::String(format!("${canonical_name}")),
        );
    }
    values
}

fn custom_property_aliases(values: &BTreeMap<String, Value>) -> Vec<(String, Value)> {
    values
        .keys()
        .map(|canonical_name| {
            (
                custom_property_alias_name(canonical_name),
                Value::String(format!("${canonical_name}")),
            )
        })
        .collect()
}

fn custom_property_alias_name(canonical_name: &str) -> String {
    let mut alias = String::with_capacity(canonical_name.len() + 2);
    alias.push_str("--");
    let mut components = canonical_name.split('.');
    if let Some(first) = components.next() {
        alias.push_str(first);
    }
    for component in components {
        alias.push('-');
        alias.push_str(component);
    }
    alias
}

#[cfg(test)]
fn custom_property_alias_name_replacing(canonical_name: &str) -> String {
    format!("--{}", canonical_name.replace('.', "-"))
}

#[cfg(test)]
fn custom_property_aliases_cloning(values: &BTreeMap<String, Value>) -> Vec<(String, Value)> {
    let canonical_names = values.keys().cloned().collect::<Vec<_>>();
    canonical_names
        .into_iter()
        .map(|canonical_name| {
            (
                format!("--{}", canonical_name.replace('.', "-")),
                Value::String(format!("${canonical_name}")),
            )
        })
        .collect()
}

// 别名解析只跟随一层 $ 引用；每个别名须直接指向规范值，循环或嵌套引用会返回 None。
pub(super) fn numeric_token_value(
    tokens: &BTreeMap<String, Value>,
    token_name: &str,
) -> Option<f32> {
    let mut value = tokens.get(token_name)?;
    for _ in 0..=1 {
        match value {
            Value::Float(value) => return Some(*value as f32),
            Value::Integer(value) => return Some(*value as f32),
            Value::String(reference) => {
                value = tokens.get(reference.strip_prefix('$')?)?;
            }
            _ => return None,
        }
    }
    None
}

pub(super) fn insert_color_token(
    values: &mut BTreeMap<String, Value>,
    name: &str,
    color: UiRgbaColor,
) {
    values.insert(name.to_string(), Value::String(color_token_hex(color)));
}

fn color_token_hex(color: UiRgbaColor) -> String {
    const COLOR_HEX_LOWER: &[u8; 16] = b"0123456789abcdef";

    let channels = color.to_u8();
    let channel_count = if channels[3] == u8::MAX { 3 } else { 4 };
    let mut encoded = String::with_capacity(1 + channel_count * 2);
    encoded.push('#');
    for &channel in &channels[..channel_count] {
        encoded.push(COLOR_HEX_LOWER[usize::from(channel >> 4)] as char);
        encoded.push(COLOR_HEX_LOWER[usize::from(channel & 0x0f)] as char);
    }
    encoded
}

pub(super) fn insert_float_token(values: &mut BTreeMap<String, Value>, name: &str, value: f32) {
    values.insert(name.to_string(), Value::Float(f64::from(value)));
}

pub(super) fn insert_integer_token(values: &mut BTreeMap<String, Value>, name: &str, value: u16) {
    values.insert(name.to_string(), Value::Integer(i64::from(value)));
}

pub(super) fn insert_string_token(values: &mut BTreeMap<String, Value>, name: &str, value: &str) {
    values.insert(name.to_string(), Value::String(value.to_string()));
}

fn canonical_cascade_token_values(tokens: &EditorDesignTokens) -> BTreeMap<String, Value> {
    let mut values = BTreeMap::new();
    tokens.palette.insert_cascade_tokens(&mut values);
    tokens.typography.insert_cascade_tokens(&mut values);
    tokens.controls.insert_cascade_tokens(&mut values);
    tokens.density.insert_density_cascade_tokens(&mut values);
    for (name, value) in tokens.chrome.cascade_entries() {
        insert_float_token(&mut values, name, value);
    }
    tokens.state_roles.insert_cascade_tokens(&mut values);
    values
}

fn legacy_density_token_aliases() -> [(&'static str, &'static str); 26] {
    [
        ("--left-drawer-width", "editor.density.left_drawer_width"),
        ("--right-drawer-width", "editor.density.right_drawer_width"),
        (
            "--bottom-output-height",
            "editor.density.bottom_output_height",
        ),
        (
            "--breakpoint-ultra-width",
            "editor.density.breakpoint_ultra_width",
        ),
        (
            "--breakpoint-narrow-width",
            "editor.density.breakpoint_narrow_width",
        ),
        (
            "--breakpoint-wide-width",
            "editor.density.breakpoint_wide_width",
        ),
        ("--compact-side-width", "editor.density.compact_side_width"),
        (
            "--ultra-compact-side-width",
            "editor.density.ultra_compact_side_width",
        ),
        (
            "--compact-left-drawer-max-width",
            "editor.density.compact_left_drawer_max_width",
        ),
        (
            "--compact-right-drawer-max-width",
            "editor.density.compact_right_drawer_max_width",
        ),
        (
            "--compact-side-min-width",
            "editor.density.compact_side_min_width",
        ),
        (
            "--minimum-document-width-fraction",
            "editor.density.minimum_document_width_fraction",
        ),
        (
            "--ultra-compact-left-drawer-max-width",
            "editor.density.ultra_compact_left_drawer_max_width",
        ),
        (
            "--ultra-compact-right-drawer-max-width",
            "editor.density.ultra_compact_right_drawer_max_width",
        ),
        (
            "--compact-bottom-available-height",
            "editor.density.compact_bottom_available_height",
        ),
        (
            "--compact-bottom-max-height",
            "editor.density.compact_bottom_max_height",
        ),
        (
            "--compact-bottom-max-available-fraction",
            "editor.density.compact_bottom_max_available_fraction",
        ),
        (
            "--compact-bottom-min-height",
            "editor.density.compact_bottom_min_height",
        ),
        (
            "--ultra-compact-bottom-available-height",
            "editor.density.ultra_compact_bottom_available_height",
        ),
        (
            "--ultra-compact-bottom-max-height",
            "editor.density.ultra_compact_bottom_max_height",
        ),
        (
            "--ultra-compact-bottom-max-available-fraction",
            "editor.density.ultra_compact_bottom_max_available_fraction",
        ),
        (
            "--ultra-compact-bottom-min-height",
            "editor.density.ultra_compact_bottom_min_height",
        ),
        (
            "--minimum-window-width",
            "editor.density.minimum_window_width",
        ),
        (
            "--minimum-window-height",
            "editor.density.minimum_window_height",
        ),
        (
            "--ultra-minimum-window-width",
            "editor.density.ultra_minimum_window_width",
        ),
        (
            "--ultra-minimum-window-height",
            "editor.density.ultra_minimum_window_height",
        ),
    ]
}

#[cfg(test)]
#[path = "tests/cascade_registry.rs"]
mod tests;

#[cfg(test)]
#[path = "cascade_registry/tests/custom_property_alias_performance_tests.rs"]
mod custom_property_alias_performance_tests;

#[cfg(test)]
#[path = "cascade_registry/tests/color_hex_performance_tests.rs"]
mod color_hex_performance_tests;
