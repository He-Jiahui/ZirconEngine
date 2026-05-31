use std::collections::BTreeMap;
use std::fs;

use toml::Value;
use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::metadata::string_token_metadata_offender;
use super::support::{collect_zui_files, editor_asset_root, runtime_asset_root};

const STYLE_SELF_KEYS: &[&str] = &[
    "background",
    "border",
    "button_color",
    "button_interaction_state",
    "button_size",
    "button_variant",
    "foreground",
    "icon_placement",
    "opacity",
];
const COLOR_TABLE_KEYS: &[&str] = &["color"];
const BORDER_TABLE_KEYS: &[&str] = &["color", "radius", "width"];

fn unknown_style_self_keys(style_self: &BTreeMap<String, Value>) -> Vec<String> {
    style_self
        .keys()
        .filter(|key| !STYLE_SELF_KEYS.contains(&key.as_str()))
        .map(|key| format!("unknown style.self key `{key}`"))
        .collect()
}

fn color_table_offenders(label: &str, value: &Value) -> Vec<String> {
    let Some(table) = value.as_table() else {
        return vec![format!("{label} is not a table")];
    };
    let mut offenders = Vec::new();
    for key in table.keys() {
        if !COLOR_TABLE_KEYS.contains(&key.as_str()) {
            offenders.push(format!("{label} declares unknown key `{key}`"));
        }
    }
    offenders.extend(color_token_offenders(label, table));
    offenders
}

fn color_token_offenders(label: &str, table: &toml::Table) -> Vec<String> {
    let Some(color_value) = table.get("color") else {
        return vec![format!("{label} declares no color")];
    };
    let Some(color) = color_value.as_str() else {
        return vec![format!("{label}.color is not a string")];
    };
    if let Some(invalid_color) = string_token_metadata_offender(color, "style color token") {
        return vec![format!("{label}.{invalid_color}")];
    }
    if !color.starts_with("material.") {
        return vec![format!(
            "{label}.color `{color}` is not a material.* theme token"
        )];
    }
    Vec::new()
}

fn border_color_offenders(value: &Value) -> Vec<String> {
    let Some(table) = value.as_table() else {
        return vec!["style.self.border is not a table".to_string()];
    };
    color_token_offenders("style.self.border", table)
}

fn border_table_offenders(value: &Value) -> Vec<String> {
    let Some(table) = value.as_table() else {
        return vec!["style.self.border is not a table".to_string()];
    };
    let mut offenders = Vec::new();
    for key in table.keys() {
        if !BORDER_TABLE_KEYS.contains(&key.as_str()) {
            offenders.push(format!("style.self.border declares unknown key `{key}`"));
        }
    }
    offenders.extend(border_color_offenders(value));
    for numeric_key in ["width", "radius"] {
        let Some(numeric_value) = table.get(numeric_key) else {
            offenders.push(format!("style.self.border declares no {numeric_key}"));
            continue;
        };
        offenders.extend(non_negative_number_offenders(
            &format!("style.self.border.{numeric_key}"),
            numeric_value,
        ));
    }
    offenders
}

fn non_negative_number_offenders(label: &str, value: &Value) -> Vec<String> {
    let Some(number) = value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
    else {
        return vec![format!("{label} is not numeric")];
    };
    if !number.is_finite() {
        return vec![format!("{label} is not finite")];
    };
    if number < 0.0 {
        return vec![format!("{label} is negative")];
    }
    Vec::new()
}

#[test]
fn production_zui_style_self_overrides_use_known_keys() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_style_tables = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                if node.style.self_values.is_empty() {
                    continue;
                }
                checked_style_tables += 1;
                let unknown_keys = unknown_style_self_keys(&node.style.self_values);
                if !unknown_keys.is_empty() {
                    offenders.push(format!(
                        "{} node `{}` style.self contains {unknown_keys:?}",
                        path.display(),
                        node_id
                    ));
                }
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_style_tables > 0,
        "production .zui assets should declare style.self overrides"
    );
    assert!(
        offenders.is_empty(),
        "production .zui style.self overrides must stay inside the runtime-recognized style key vocabulary: {offenders:#?}"
    );
}

#[test]
fn production_zui_style_color_tables_use_material_tokens() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_color_tables = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                for color_key in ["background", "foreground"] {
                    let Some(color_table) = node.style.self_values.get(color_key) else {
                        continue;
                    };
                    checked_color_tables += 1;
                    let invalid_colors =
                        color_table_offenders(&format!("style.self.{color_key}"), color_table);
                    if !invalid_colors.is_empty() {
                        offenders.push(format!(
                            "{} node `{}` contains {invalid_colors:?}",
                            path.display(),
                            node_id
                        ));
                    }
                }
                if let Some(border) = node.style.self_values.get("border") {
                    checked_color_tables += 1;
                    let invalid_border = border_color_offenders(border);
                    if !invalid_border.is_empty() {
                        offenders.push(format!(
                            "{} node `{}` contains {invalid_border:?}",
                            path.display(),
                            node_id
                        ));
                    }
                }
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_color_tables > 0,
        "production .zui style overrides should declare theme color tables"
    );
    assert!(
        offenders.is_empty(),
        "production .zui style color tables must use clean material.* theme tokens for Style Inspector and theme hot reload: {offenders:#?}"
    );
}

#[test]
fn production_zui_style_numeric_overrides_are_non_negative() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_numeric_values = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                if let Some(opacity) = node.style.self_values.get("opacity") {
                    checked_numeric_values += 1;
                    let invalid_opacity =
                        non_negative_number_offenders("style.self.opacity", opacity);
                    if !invalid_opacity.is_empty() {
                        offenders.push(format!(
                            "{} node `{}` contains {invalid_opacity:?}",
                            path.display(),
                            node_id
                        ));
                    }
                }
                let Some(border) = node.style.self_values.get("border") else {
                    continue;
                };
                checked_numeric_values += 2;
                let invalid_border = border_table_offenders(border);
                if !invalid_border.is_empty() {
                    offenders.push(format!(
                        "{} node `{}` contains {invalid_border:?}",
                        path.display(),
                        node_id
                    ));
                }
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_numeric_values > 0,
        "production .zui style overrides should declare numeric style values"
    );
    assert!(
        offenders.is_empty(),
        "production .zui style numeric overrides must be finite and non-negative for stable render/style inspection: {offenders:#?}"
    );
}
