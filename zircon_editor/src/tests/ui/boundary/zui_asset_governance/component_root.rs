use std::fs;

use toml::Value;
use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::metadata::string_token_metadata_offender;
use super::support::{collect_zui_files, editor_asset_root, runtime_asset_root};

fn layout_width_stretch_offender(width: Option<&Value>) -> Option<String> {
    let Some(width) = width else {
        return Some("declares no width contract".to_string());
    };
    let Some(width_table) = width.as_table() else {
        return Some("declares non-table width metadata".to_string());
    };
    let Some(stretch_value) = width_table.get("stretch") else {
        return Some("declares no width stretch mode".to_string());
    };
    let Some(stretch) = stretch_value.as_str() else {
        return Some("declares non-string width stretch mode".to_string());
    };
    string_token_metadata_offender(stretch, "width stretch mode")
}

#[test]
fn production_zui_component_roots_are_concrete_authoring_nodes() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_components = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (component_name, component) in &document.components {
                checked_components += 1;
                let Some(root_node) = document.nodes.get(&component.root) else {
                    offenders.push(format!(
                        "{} component `{}` references missing root node `{}`",
                        path.display(),
                        component_name,
                        component.root
                    ));
                    continue;
                };

                if root_node.component.trim() == "Slot" {
                    offenders.push(format!(
                        "{} component `{}` uses Slot placeholder `{}` as its root",
                        path.display(),
                        component_name,
                        component.root
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
        checked_components > 0,
        "production .zui assets should declare component contracts"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component roots must resolve to concrete non-Slot authoring nodes: {offenders:#?}"
    );
}

#[test]
fn production_zui_component_root_nodes_declare_layout_metadata() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_components = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (component_name, component) in &document.components {
                checked_components += 1;
                let Some(root_node) = document.nodes.get(&component.root) else {
                    continue;
                };

                if root_node
                    .layout
                    .as_ref()
                    .is_none_or(|layout| layout.is_empty())
                {
                    offenders.push(format!(
                        "{} component `{}` root node `{}` declares no layout metadata",
                        path.display(),
                        component_name,
                        component.root
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
        checked_components > 0,
        "production .zui assets should declare component contracts"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component roots must declare explicit layout metadata for previews, hierarchy editing, and surface mounting: {offenders:#?}"
    );
}

#[test]
fn production_zui_component_root_layouts_declare_width_contract() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_components = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (component_name, component) in &document.components {
                checked_components += 1;
                let Some(root_node) = document.nodes.get(&component.root) else {
                    continue;
                };
                let Some(layout) = root_node.layout.as_ref() else {
                    offenders.push(format!(
                        "{} component `{}` root node `{}` declares no layout metadata",
                        path.display(),
                        component_name,
                        component.root
                    ));
                    continue;
                };

                if !layout.contains_key("width") {
                    offenders.push(format!(
                        "{} component `{}` root node `{}` layout declares no width contract",
                        path.display(),
                        component_name,
                        component.root
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
        checked_components > 0,
        "production .zui assets should declare component contracts"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component root layouts must declare an explicit width contract for stable preview sizing and surface mounting: {offenders:#?}"
    );
}

#[test]
fn production_zui_component_root_width_contracts_declare_stretch_modes() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_components = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (component_name, component) in &document.components {
                checked_components += 1;
                let Some(root_node) = document.nodes.get(&component.root) else {
                    continue;
                };
                let width = root_node
                    .layout
                    .as_ref()
                    .and_then(|layout| layout.get("width"));
                if let Some(invalid_width) = layout_width_stretch_offender(width) {
                    offenders.push(format!(
                        "{} component `{}` root node `{}` {invalid_width}",
                        path.display(),
                        component_name,
                        component.root
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
        checked_components > 0,
        "production .zui assets should declare component contracts"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component root width contracts must declare clean stretch modes for preview and host sizing semantics: {offenders:#?}"
    );
}
