use std::collections::BTreeMap;

use toml::Value;

use crate::ui::style::UiRgbaColor;

use super::EditorDesignTokens;

pub(super) fn cascade_token_values(tokens: &EditorDesignTokens) -> BTreeMap<String, Value> {
    let mut values = canonical_cascade_token_values(tokens);
    let canonical_names = values.keys().cloned().collect::<Vec<_>>();
    for canonical_name in canonical_names {
        let custom_property_name = format!("--{}", canonical_name.replace('.', "-"));
        values.insert(
            custom_property_name,
            Value::String(format!("${canonical_name}")),
        );
    }

    for (legacy_name, canonical_name) in legacy_density_token_aliases() {
        values.insert(
            legacy_name.to_string(),
            Value::String(format!("${canonical_name}")),
        );
    }
    values
}

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
    let [red, green, blue, alpha] = color.to_u8();
    let value = if alpha == u8::MAX {
        format!("#{red:02x}{green:02x}{blue:02x}")
    } else {
        format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
    };
    values.insert(name.to_string(), Value::String(value));
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
mod tests {
    use super::canonical_cascade_token_values;
    use crate::ui::design_tokens::EditorDesignTokens;

    #[test]
    fn workbench_chrome_metrics_are_registered_as_logical_float_tokens() {
        let values = canonical_cascade_token_values(&EditorDesignTokens::workbench_dark());

        for (name, expected) in [
            ("editor.chrome.top_bar.height", 25.0),
            ("editor.chrome.host_bar.height", 32.0),
            ("editor.chrome.status_bar.height", 24.0),
            ("editor.chrome.panel_header.height", 25.0),
            ("editor.chrome.document_header.height", 31.0),
            ("editor.chrome.viewport_toolbar.height", 28.0),
            ("editor.chrome.activity_rail.width", 34.0),
            ("editor.chrome.separator.thickness", 1.0),
            ("editor.chrome.splitter.hit_size", 8.0),
        ] {
            assert_eq!(
                values.get(name).and_then(toml::Value::as_float),
                Some(expected),
                "missing or invalid Workbench chrome token `{name}`"
            );
        }
    }

    #[test]
    fn workbench_dark_palette_matches_component_prototype_baseline() {
        let values = canonical_cascade_token_values(&EditorDesignTokens::workbench_dark());

        for (name, expected) in [
            ("editor.surface.0", "#090f12"),
            ("editor.surface.1", "#0f171b"),
            ("editor.surface.2", "#10181c"),
            ("editor.surface.3", "#141d22"),
            ("editor.surface.recessed", "#081014"),
            ("editor.surface.hover", "#1a252b"),
            ("editor.surface.selected", "#164349"),
            ("editor.accent", "#35c7d0"),
            ("editor.border", "#223037"),
            ("editor.separator.strong", "#223037"),
            ("editor.separator.soft", "#1b2428"),
            ("editor.text.primary", "#d9e5e8"),
            ("editor.text.secondary", "#9ca9ad"),
            ("editor.text.disabled", "#69787e"),
            ("editor.popup", "#141d22"),
            ("editor.track", "#1a252b"),
            ("editor.focus.ring", "#35c7d0"),
        ] {
            assert_eq!(
                values.get(name).and_then(toml::Value::as_str),
                Some(expected),
                "palette token `{name}` diverged from the component prototype baseline"
            );
        }
    }
}
