use std::collections::BTreeMap;
use std::fs;

use toml::Value;
use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::metadata::string_token_metadata_offender;
use super::support::{collect_zui_files, editor_asset_root, runtime_asset_root};

const VALID_AXIS_STRETCH_MODES: &[&str] = &["Fixed", "Stretch"];
const AXIS_SIZE_KEYS: &[&str] = &["min", "preferred", "max"];

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

fn axis_stretch_mode_offender(axis: &str, axis_table: &toml::Table) -> Option<String> {
    let stretch_value = axis_table.get("stretch")?;
    let Some(stretch) = stretch_value.as_str() else {
        return Some(format!("layout.{axis}.stretch is not a string"));
    };
    if let Some(invalid_stretch) = string_token_metadata_offender(stretch, "axis stretch mode") {
        return Some(format!("layout.{axis}.{invalid_stretch}"));
    }
    (!VALID_AXIS_STRETCH_MODES.contains(&stretch)).then(|| {
        format!(
            "layout.{axis}.stretch declares unsupported mode `{stretch}`; expected one of {VALID_AXIS_STRETCH_MODES:?}"
        )
    })
}

fn axis_declares_fixed_stretch(axis_table: &toml::Table) -> bool {
    axis_table.get("stretch").and_then(Value::as_str) == Some("Fixed")
}

fn axis_declares_size_contract(axis_table: &toml::Table) -> bool {
    AXIS_SIZE_KEYS
        .iter()
        .any(|key| axis_table.contains_key(*key))
}

fn axis_declares_stretch(axis_table: &toml::Table) -> bool {
    axis_table.get("stretch").and_then(Value::as_str) == Some("Stretch")
}

fn numeric_axis_value(
    axis: &str,
    axis_table: &toml::Table,
    key: &str,
) -> Result<Option<f64>, String> {
    let Some(value) = axis_table.get(key) else {
        return Ok(None);
    };
    let Some(number) = value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
    else {
        return Err(format!("layout.{axis}.{key} is not numeric"));
    };
    if !number.is_finite() {
        return Err(format!("layout.{axis}.{key} is not finite"));
    }
    if number < 0.0 {
        return Err(format!("layout.{axis}.{key} is negative"));
    }
    Ok(Some(number))
}

fn axis_ordering_offenders(axis: &str, axis_table: &toml::Table) -> Vec<String> {
    let mut offenders = Vec::new();
    let min = match numeric_axis_value(axis, axis_table, "min") {
        Ok(value) => value,
        Err(offender) => {
            offenders.push(offender);
            None
        }
    };
    let preferred = match numeric_axis_value(axis, axis_table, "preferred") {
        Ok(value) => value,
        Err(offender) => {
            offenders.push(offender);
            None
        }
    };
    let max = match numeric_axis_value(axis, axis_table, "max") {
        Ok(value) => value,
        Err(offender) => {
            offenders.push(offender);
            None
        }
    };

    if let (Some(min), Some(preferred)) = (min, preferred) {
        if min > preferred {
            offenders.push(format!(
                "layout.{axis}.min {min} is greater than preferred {preferred}"
            ));
        }
    }
    if let (Some(preferred), Some(max)) = (preferred, max) {
        if preferred > max {
            offenders.push(format!(
                "layout.{axis}.preferred {preferred} is greater than max {max}"
            ));
        }
    }
    if let (Some(min), Some(max)) = (min, max) {
        if min > max {
            offenders.push(format!("layout.{axis}.min {min} is greater than max {max}"));
        }
    }
    offenders
}

#[test]
fn production_zui_axis_layout_stretch_modes_are_known() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_axis_tables = 0usize;
    let mut checked_stretch_modes = 0usize;
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
                for axis in ["width", "height"] {
                    let Some(axis_table) = axis_layout_table(layout, axis) else {
                        continue;
                    };
                    checked_axis_tables += 1;
                    let axis_table = match axis_table {
                        Ok(axis_table) => axis_table,
                        Err(offender) => {
                            offenders.push(format!(
                                "{} node `{}` {offender}",
                                path.display(),
                                node_id
                            ));
                            continue;
                        }
                    };
                    if axis_table.contains_key("stretch") {
                        checked_stretch_modes += 1;
                    }
                    if let Some(offender) = axis_stretch_mode_offender(axis, axis_table) {
                        offenders.push(format!("{} node `{}` {offender}", path.display(), node_id));
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
        checked_axis_tables > 0 && checked_stretch_modes > 0,
        "production .zui assets should declare axis layout stretch metadata"
    );
    assert!(
        offenders.is_empty(),
        "production .zui axis layout stretch modes must use known authoring values: {offenders:#?}"
    );
}

#[test]
fn production_zui_fixed_axis_layouts_declare_size_contracts() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_axis_tables = 0usize;
    let mut checked_fixed_axis_tables = 0usize;
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
                for axis in ["width", "height"] {
                    let Some(axis_table) = axis_layout_table(layout, axis) else {
                        continue;
                    };
                    checked_axis_tables += 1;
                    let axis_table = match axis_table {
                        Ok(axis_table) => axis_table,
                        Err(offender) => {
                            offenders.push(format!(
                                "{} node `{}` {offender}",
                                path.display(),
                                node_id
                            ));
                            continue;
                        }
                    };
                    if !axis_declares_fixed_stretch(axis_table) {
                        continue;
                    }
                    checked_fixed_axis_tables += 1;
                    if !axis_declares_size_contract(axis_table) {
                        offenders.push(format!(
                            "{} node `{}` layout.{axis} declares Fixed stretch without any of {AXIS_SIZE_KEYS:?}",
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
        checked_axis_tables > 0 && checked_fixed_axis_tables > 0,
        "production .zui assets should declare fixed axis layout metadata"
    );
    assert!(
        offenders.is_empty(),
        "production .zui Fixed axis layouts must declare at least one numeric size contract: {offenders:#?}"
    );
}

#[test]
fn production_zui_axis_layout_stretch_semantics_are_unambiguous() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_axis_tables = 0usize;
    let mut checked_stretched_axis_tables = 0usize;
    let mut checked_fixed_axis_tables = 0usize;
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
                for axis in ["width", "height"] {
                    let Some(axis_table) = axis_layout_table(layout, axis) else {
                        continue;
                    };
                    checked_axis_tables += 1;
                    let axis_table = match axis_table {
                        Ok(axis_table) => axis_table,
                        Err(offender) => {
                            offenders.push(format!(
                                "{} node `{}` {offender}",
                                path.display(),
                                node_id
                            ));
                            continue;
                        }
                    };
                    if axis_declares_stretch(axis_table) {
                        checked_stretched_axis_tables += 1;
                        let declared_size_keys = AXIS_SIZE_KEYS
                            .iter()
                            .filter(|key| axis_table.contains_key(**key))
                            .copied()
                            .collect::<Vec<_>>();
                        if !declared_size_keys.is_empty() {
                            offenders.push(format!(
                                "{} node `{}` layout.{axis} declares Stretch with size keys {declared_size_keys:?}",
                                path.display(),
                                node_id
                            ));
                        }
                    }
                    if axis_declares_fixed_stretch(axis_table) {
                        checked_fixed_axis_tables += 1;
                        if !axis_table.contains_key("preferred") {
                            offenders.push(format!(
                                "{} node `{}` layout.{axis} declares Fixed without preferred size",
                                path.display(),
                                node_id
                            ));
                        }
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
        checked_axis_tables > 0
            && checked_stretched_axis_tables > 0
            && checked_fixed_axis_tables > 0,
        "production .zui assets should declare both Stretch and Fixed axis layout metadata"
    );
    assert!(
        offenders.is_empty(),
        "production .zui axis layout semantics must stay unambiguous: Stretch axes carry no explicit size constraints and Fixed axes declare preferred size for resize handles and Inspector controls: {offenders:#?}"
    );
}

#[test]
fn production_zui_axis_layout_numeric_ranges_are_ordered() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_axis_tables = 0usize;
    let mut checked_numeric_axis_tables = 0usize;
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
                for axis in ["width", "height"] {
                    let Some(axis_table) = axis_layout_table(layout, axis) else {
                        continue;
                    };
                    checked_axis_tables += 1;
                    let axis_table = match axis_table {
                        Ok(axis_table) => axis_table,
                        Err(offender) => {
                            offenders.push(format!(
                                "{} node `{}` {offender}",
                                path.display(),
                                node_id
                            ));
                            continue;
                        }
                    };
                    if axis_declares_size_contract(axis_table) {
                        checked_numeric_axis_tables += 1;
                    }
                    let invalid_ranges = axis_ordering_offenders(axis, axis_table);
                    if !invalid_ranges.is_empty() {
                        offenders.push(format!(
                            "{} node `{}` contains invalid axis ranges {invalid_ranges:?}",
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
        checked_axis_tables > 0 && checked_numeric_axis_tables > 0,
        "production .zui assets should declare numeric axis layout constraints"
    );
    assert!(
        offenders.is_empty(),
        "production .zui numeric axis layout constraints must be finite, non-negative, and ordered as min <= preferred <= max: {offenders:#?}"
    );
}
