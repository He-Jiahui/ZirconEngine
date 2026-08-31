use std::collections::BTreeMap;

use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetReference, AssetUri};
use crate::ui::surface::UiSurface;
use crate::ui::template::{
    UiAssetDependencyIndex, UiAssetHotReloadPlan, UiAssetLoader, UiAssetNodeTarget,
    UiAssetSurfaceIndex, UiDocumentCompiler,
};
use toml::Value;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use zircon_runtime_interface::ui::layout::{AxisConstraint, BoxConstraints, StretchMode, UiSize};
use zircon_runtime_interface::ui::tree::{
    UiDirtyFlags, UiTemplateNodeMetadata, UiTree, UiTreeNode,
};

const TEMPLATE_WITH_RESOURCES: &str = r#"
[asset]
kind = "layout"
id = "res://ui/views/main.zui"
version = 1

[imports]
resources = [
  { kind = "font", uri = "res://fonts/inter.font.toml", fallback = { mode = "placeholder", uri = "res://fonts/system.ttf" } },
  { kind = "image", uri = "res://ui/icons/run.svg", fallback = { mode = "optional" } },
]

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { text = "Main" }
"#;

mod binding_ownership;
mod binding_ownership_performance;
mod dirty_targets;
mod node_resources;
mod surface_edges;

fn asset_ref(value: &str) -> AssetReference {
    AssetReference::from_locator(uri(value))
}

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

fn tree_id(value: &str) -> UiTreeId {
    UiTreeId::new(value)
}

fn dirty_test_surface(tree_id: &UiTreeId) -> UiSurface {
    let mut surface = UiSurface::new(tree_id.clone());
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_constraints(
            BoxConstraints {
                width: fixed_constraint(120.0),
                height: fixed_constraint(60.0),
            },
        ),
    );
    surface.compute_layout(UiSize::new(120.0, 60.0)).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn dirty_test_surface_with_nodes(tree_id: &UiTreeId) -> UiSurface {
    let mut surface = dirty_test_surface(tree_id);
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/icon")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(20.0),
                    height: fixed_constraint(20.0),
                },
            ),
        )
        .unwrap();
    surface.compute_layout(UiSize::new(120.0, 60.0)).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn resource_value(kind: &str, uri: &str, fallback: Option<(&str, &str)>) -> Value {
    let mut table = toml::map::Map::new();
    table.insert("kind".to_string(), Value::String(kind.to_string()));
    table.insert("uri".to_string(), Value::String(uri.to_string()));
    if let Some((mode, fallback_uri)) = fallback {
        let mut fallback = toml::map::Map::new();
        fallback.insert("mode".to_string(), Value::String(mode.to_string()));
        fallback.insert("uri".to_string(), Value::String(fallback_uri.to_string()));
        table.insert("fallback".to_string(), Value::Table(fallback));
    }
    Value::Table(table)
}

#[test]
fn surface_index_target_dedupe_borrows_index_entries_and_counts_in_one_pass() {
    let source = include_str!("../template/asset/surface_index.rs");

    assert!(source.contains("seen: &mut BTreeSet<&'a UiTreeId>"));
    assert!(source.contains("seen: &mut BTreeSet<&'a UiAssetNodeTarget>"));
    assert!(!source.contains("seen.insert(surface.clone())"));
    assert!(!source.contains("seen.insert(target.clone())"));
    assert!(source.contains("expected_target_count += push_nodes_for_surface("));
    assert!(!source.contains("fn target_surface_count("));
}
