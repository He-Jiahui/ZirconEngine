use std::collections::BTreeMap;
use std::fs;

use toml::Value;
use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::metadata::string_token_metadata_offender;
use super::support::{collect_zui_files, editor_asset_root, runtime_asset_root};

const VALID_CONTAINER_KINDS: &[&str] = &["HorizontalBox", "Masonry", "VerticalBox"];
const LAYOUT_KEYS: &[&str] = &["clip", "container", "height", "width"];
const AXIS_LAYOUT_KEYS: &[&str] = &["max", "min", "preferred", "stretch"];
const CONTAINER_LAYOUT_KEYS: &[&str] = &["columns", "gap", "kind", "sequential"];

fn axis_layout_table<'a>(
    layout: &'a BTreeMap<String, Value>,
    axis: &str,
) -> Option<Result<&'a toml::Table, String>> {
    layout.get(axis).map(|axis_value| {
        axis_value
            .as_table()
            .ok_or_else(|| format!("layout.{axis} is not a table"))
    })
}

fn container_layout_offenders(container: &Value) -> Vec<String> {
    let Some(container_table) = container.as_table() else {
        return vec!["layout.container is not a table".to_string()];
    };

    let mut offenders = Vec::new();
    let Some(kind_value) = container_table.get("kind") else {
        offenders.push("layout.container declares no kind".to_string());
        return offenders;
    };
    let Some(kind) = kind_value.as_str() else {
        offenders.push("layout.container.kind is not a string".to_string());
        return offenders;
    };
    if let Some(invalid_kind) = string_token_metadata_offender(kind, "container kind") {
        offenders.push(format!("layout.container.{invalid_kind}"));
    } else if !VALID_CONTAINER_KINDS.contains(&kind) {
        offenders.push(format!(
            "layout.container.kind declares unsupported kind `{kind}`; expected one of {VALID_CONTAINER_KINDS:?}"
        ));
    } else if kind == "Masonry" {
        offenders.extend(masonry_container_offenders(container_table));
    }

    if let Some(gap_value) = container_table.get("gap") {
        let Some(gap) = gap_value
            .as_float()
            .or_else(|| gap_value.as_integer().map(|value| value as f64))
        else {
            offenders.push("layout.container.gap is not numeric".to_string());
            return offenders;
        };
        if !gap.is_finite() {
            offenders.push("layout.container.gap is not finite".to_string());
        } else if gap < 0.0 {
            offenders.push("layout.container.gap is negative".to_string());
        }
    }

    offenders
}

fn masonry_container_offenders(container_table: &toml::Table) -> Vec<String> {
    let mut offenders = Vec::new();

    let Some(columns_value) = container_table.get("columns") else {
        offenders.push("layout.container.columns is required for Masonry".to_string());
        return offenders;
    };
    let Some(columns) = columns_value.as_integer() else {
        offenders.push("layout.container.columns is not an integer".to_string());
        return offenders;
    };
    if columns <= 0 {
        offenders.push("layout.container.columns is not positive".to_string());
    }

    if let Some(sequential_value) = container_table.get("sequential") {
        if sequential_value.as_bool().is_none() {
            offenders.push("layout.container.sequential is not a boolean".to_string());
        }
    }

    offenders
}

fn layout_clip_offender(layout: &BTreeMap<String, Value>) -> Option<String> {
    let clip_value = layout.get("clip")?;
    clip_value
        .as_bool()
        .is_none()
        .then(|| "layout.clip is not a boolean".to_string())
}

fn unknown_layout_key_offenders(node_id: &str, layout: &BTreeMap<String, Value>) -> Vec<String> {
    let mut offenders = Vec::new();

    for key in layout.keys() {
        if !LAYOUT_KEYS.contains(&key.as_str()) {
            offenders.push(format!(
                "node `{node_id}` declares unknown layout key `{key}`"
            ));
        }
    }

    for axis in ["width", "height"] {
        let Some(axis_table) = axis_layout_table(layout, axis) else {
            continue;
        };
        let Ok(axis_table) = axis_table else {
            continue;
        };
        for key in axis_table.keys() {
            if !AXIS_LAYOUT_KEYS.contains(&key.as_str()) {
                offenders.push(format!(
                    "node `{node_id}` declares unknown layout.{axis} key `{key}`"
                ));
            }
        }
    }

    let Some(container_table) = layout.get("container").and_then(Value::as_table) else {
        return offenders;
    };
    for key in container_table.keys() {
        if !CONTAINER_LAYOUT_KEYS.contains(&key.as_str()) {
            offenders.push(format!(
                "node `{node_id}` declares unknown layout.container key `{key}`"
            ));
        }
    }

    offenders
}

#[test]
fn production_zui_layout_metadata_uses_known_keys() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_layout_tables = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                let Some(layout) = &node.layout else {
                    continue;
                };
                checked_layout_tables += 1;
                let invalid_keys = unknown_layout_key_offenders(node_id, layout);
                if !invalid_keys.is_empty() {
                    offenders.push(format!("{} {invalid_keys:?}", path.display()));
                }
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_layout_tables > 0,
        "production .zui assets should declare layout metadata"
    );
    assert!(
        offenders.is_empty(),
        "production .zui layout metadata must stay inside known shared layout keys so runtime layout, retained-host projection, and Widget Editor Inspector interpret the same fields: {offenders:#?}"
    );
}

#[test]
fn production_zui_layout_clip_metadata_is_boolean() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_layout_tables = 0usize;
    let mut checked_clip_values = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                let Some(layout) = &node.layout else {
                    continue;
                };
                checked_layout_tables += 1;
                if layout.contains_key("clip") {
                    checked_clip_values += 1;
                }
                if let Some(offender) = layout_clip_offender(layout) {
                    offenders.push(format!("{} node `{}` {offender}", path.display(), node_id));
                }
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_layout_tables > 0 && checked_clip_values > 0,
        "production .zui assets should include explicit clip layout metadata"
    );
    assert!(
        offenders.is_empty(),
        "production .zui layout clip metadata must be boolean when present: {offenders:#?}"
    );
}

#[test]
fn production_zui_container_layout_metadata_is_known_and_non_negative() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_layout_tables = 0usize;
    let mut checked_container_tables = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                let Some(layout) = &node.layout else {
                    continue;
                };
                checked_layout_tables += 1;
                let Some(container) = layout.get("container") else {
                    continue;
                };
                checked_container_tables += 1;
                let invalid_container = container_layout_offenders(container);
                if !invalid_container.is_empty() {
                    offenders.push(format!(
                        "{} node `{}` contains invalid container metadata {invalid_container:?}",
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
        checked_layout_tables > 0 && checked_container_tables > 0,
        "production .zui assets should declare container layout metadata"
    );
    assert!(
        offenders.is_empty(),
        "production .zui container layout metadata must use known kinds and non-negative numeric spacing: {offenders:#?}"
    );
}
