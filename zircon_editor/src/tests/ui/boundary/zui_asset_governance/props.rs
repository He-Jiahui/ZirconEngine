use std::collections::BTreeMap;
use std::fs;

use toml::Value;
use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::metadata::string_metadata_offender;
use super::support::{collect_zui_files, editor_asset_root, runtime_asset_root};

fn is_authoring_prop_key(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn prop_key_offenders(props: &BTreeMap<String, Value>) -> Vec<String> {
    let mut offenders = Vec::new();
    collect_prop_key_offenders(props.iter(), "props", &mut offenders);
    offenders
}

fn collect_prop_key_offenders<'a>(
    props: impl IntoIterator<Item = (&'a String, &'a Value)>,
    path: &str,
    offenders: &mut Vec<String>,
) {
    for (key, value) in props {
        let key_path = format!("{path}.{key}");
        if let Some(invalid_key) = string_metadata_offender(key, "prop key") {
            offenders.push(format!("{key_path} declares {invalid_key}"));
        } else if !is_authoring_prop_key(key) {
            offenders.push(format!(
                "{key_path} uses a non-authoring-safe prop key `{key}`"
            ));
        }

        let Some(table) = value.as_table() else {
            continue;
        };
        collect_prop_key_offenders(table, &key_path, offenders);
    }
}

fn prop_value_offenders(props: &BTreeMap<String, Value>) -> Vec<String> {
    let mut offenders = Vec::new();
    for (key, value) in props {
        collect_prop_value_offenders(value, &format!("props.{key}"), &mut offenders);
    }
    offenders
}

fn prop_string_value_offender(value: &str) -> Option<String> {
    (value.trim() != value).then(|| format!("whitespace-padded prop string value `{value}`"))
}

fn collect_prop_value_offenders(value: &Value, path: &str, offenders: &mut Vec<String>) {
    match value {
        Value::String(text) => {
            if let Some(invalid_string) = prop_string_value_offender(text) {
                offenders.push(format!("{path} declares {invalid_string}"));
            }
        }
        Value::Integer(_) | Value::Boolean(_) => {}
        Value::Float(number) => {
            if !number.is_finite() {
                offenders.push(format!("{path} is not finite"));
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                collect_prop_value_offenders(value, &format!("{path}[{index}]"), offenders);
            }
        }
        Value::Table(table) => {
            for (key, value) in table {
                collect_prop_value_offenders(value, &format!("{path}.{key}"), offenders);
            }
        }
        Value::Datetime(_) => {
            offenders.push(format!("{path} uses TOML datetime instead of a string"));
        }
    }
}

#[test]
fn production_zui_prop_keys_are_authoring_safe() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_prop_tables = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                if node.props.is_empty() {
                    continue;
                }
                checked_prop_tables += 1;
                let invalid_keys = prop_key_offenders(&node.props);
                if !invalid_keys.is_empty() {
                    offenders.push(format!(
                        "{} node `{}` contains {invalid_keys:?}",
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
        checked_prop_tables > 0,
        "production .zui assets should declare node props"
    );
    assert!(
        offenders.is_empty(),
        "production .zui prop keys must remain stable ASCII authoring identifiers for Inspector addressing and host projection: {offenders:#?}"
    );
}

#[test]
fn production_zui_prop_values_are_inspector_serializable() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_prop_tables = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                if node.props.is_empty() {
                    continue;
                }
                checked_prop_tables += 1;
                let invalid_values = prop_value_offenders(&node.props);
                if !invalid_values.is_empty() {
                    offenders.push(format!(
                        "{} node `{}` contains {invalid_values:?}",
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
        checked_prop_tables > 0,
        "production .zui assets should declare node props"
    );
    assert!(
        offenders.is_empty(),
        "production .zui prop values must stay inside Inspector-serializable TOML primitives, arrays, and tables with trim-stable strings and finite numbers: {offenders:#?}"
    );
}
