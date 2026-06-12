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
id = "res://ui/views/main.v2.ui.toml"
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

#[test]
fn surface_index_tracks_assets_and_replaces_stale_surface_edges() {
    let mut index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    let inspector = tree_id("runtime.ui.inspector");

    index.record_surface_assets(
        main.clone(),
        [
            "res://ui/views/main.v2.ui.toml",
            "res://ui/theme/base.theme.toml",
            "res://ui/theme/base.theme.toml",
        ],
    );
    index.record_surface_assets(
        inspector.clone(),
        [
            "res://ui/views/inspector.v2.ui.toml",
            "res://ui/theme/base.theme.toml",
        ],
    );

    assert_eq!(index.surface_count(), 2);
    assert_eq!(
        index.assets_for_surface(&main),
        &[
            "res://ui/views/main.v2.ui.toml".to_string(),
            "res://ui/theme/base.theme.toml".to_string(),
        ]
    );
    assert_eq!(
        index
            .surfaces_for_asset("res://ui/theme/base.theme.toml")
            .cloned()
            .collect::<Vec<_>>(),
        vec![inspector.clone(), main.clone()]
    );

    index.record_surface_assets(main.clone(), ["res://ui/theme/updated.theme.toml"]);

    assert_eq!(
        index
            .surfaces_for_asset("res://ui/theme/base.theme.toml")
            .cloned()
            .collect::<Vec<_>>(),
        vec![inspector]
    );
    assert_eq!(
        index
            .surfaces_for_asset("res://ui/theme/updated.theme.toml")
            .cloned()
            .collect::<Vec<_>>(),
        vec![main]
    );
}

#[test]
fn surface_index_records_compiled_document_resource_dependencies() {
    let document = UiAssetLoader::load_toml_str(TEMPLATE_WITH_RESOURCES).unwrap();
    let compiler = UiDocumentCompiler::default();
    let compiled = compiler.compile(&document).unwrap();
    let mut index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");

    index.record_compiled_surface(main.clone(), &compiled);

    assert_eq!(
        index.assets_for_surface(&main),
        &[
            "res://ui/views/main.v2.ui.toml".to_string(),
            "res://fonts/inter.font.toml".to_string(),
            "res://fonts/system.ttf".to_string(),
            "res://ui/icons/run.svg".to_string(),
        ]
    );
    assert_eq!(
        index
            .surfaces_for_asset("res://ui/icons/run.svg")
            .cloned()
            .collect::<Vec<_>>(),
        vec![main]
    );
}

#[test]
fn hot_reload_plan_maps_template_theme_and_resource_targets_to_surfaces() {
    let mut dependency_index = UiAssetDependencyIndex::new();
    dependency_index.record_compiled(
        "res://ui/views/main.v2.ui.toml",
        &[
            asset_ref("res://ui/theme/base.theme.toml"),
            asset_ref("res://ui/icons/run.svg"),
        ],
    );
    dependency_index.record_compiled(
        "res://ui/views/inspector.v2.ui.toml",
        &[asset_ref("res://ui/theme/base.theme.toml")],
    );

    let mut surface_index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    let inspector = tree_id("runtime.ui.inspector");
    surface_index.record_surface_assets(
        main.clone(),
        [
            "res://ui/views/main.v2.ui.toml",
            "res://ui/theme/base.theme.toml",
            "res://ui/icons/run.svg",
        ],
    );
    surface_index.record_surface_assets(
        inspector.clone(),
        [
            "res://ui/views/inspector.v2.ui.toml",
            "res://ui/theme/base.theme.toml",
        ],
    );

    let theme_report = dependency_index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Modified,
        uri("res://ui/theme/base.theme.toml"),
        None,
    )]);
    let theme_plan = UiAssetHotReloadPlan::from_watch_report(&theme_report);
    let theme_targets = surface_index.target_surfaces_for_plan(&theme_plan);
    assert_eq!(
        theme_targets.theme_restyle_surfaces,
        vec![inspector.clone(), main.clone()]
    );
    assert!(theme_targets.template_rebuild_surfaces.is_empty());
    assert!(theme_targets.dirty.style);
    assert!(!theme_targets.rebuild_required);

    let icon_report = dependency_index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Modified,
        uri("res://ui/icons/run.svg"),
        None,
    )]);
    let icon_plan = UiAssetHotReloadPlan::from_watch_report(&icon_report);
    let icon_targets = surface_index.target_surfaces_for_plan(&icon_plan);
    assert_eq!(icon_targets.resource_damage_surfaces, vec![main]);
    assert_eq!(
        icon_targets.dirty,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );
}

#[test]
fn surface_index_tracks_node_asset_edges_and_replaces_stale_node_edges() {
    let mut index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    let run_button = UiNodeId::new(2);
    let status = UiNodeId::new(3);

    index.record_surface_assets(
        main.clone(),
        [
            "res://ui/views/main.v2.ui.toml",
            "res://ui/icons/run.svg",
            "res://fonts/inter.font.toml",
        ],
    );
    index.record_node_assets(
        main.clone(),
        run_button,
        [
            "res://ui/icons/run.svg",
            "res://ui/icons/run.svg",
            "res://fonts/inter.font.toml",
        ],
    );
    index.record_node_assets(main.clone(), status, ["res://fonts/inter.font.toml"]);

    assert_eq!(
        index.assets_for_node(&main, run_button),
        &[
            "res://ui/icons/run.svg".to_string(),
            "res://fonts/inter.font.toml".to_string(),
        ]
    );
    assert_eq!(
        index
            .nodes_for_asset("res://fonts/inter.font.toml")
            .cloned()
            .collect::<Vec<_>>(),
        vec![
            UiAssetNodeTarget {
                tree_id: main.clone(),
                node_id: run_button,
            },
            UiAssetNodeTarget {
                tree_id: main.clone(),
                node_id: status,
            },
        ]
    );

    index.record_node_assets(main.clone(), run_button, ["res://ui/icons/save.svg"]);

    assert_eq!(
        index
            .nodes_for_asset("res://ui/icons/run.svg")
            .cloned()
            .collect::<Vec<_>>(),
        Vec::<UiAssetNodeTarget>::new()
    );
    assert_eq!(
        index
            .nodes_for_asset("res://ui/icons/save.svg")
            .cloned()
            .collect::<Vec<_>>(),
        vec![UiAssetNodeTarget {
            tree_id: main.clone(),
            node_id: run_button,
        }]
    );

    index.remove_surface(&main);

    assert!(index.assets_for_node(&main, status).is_empty());
    assert!(index
        .nodes_for_asset("res://fonts/inter.font.toml")
        .next()
        .is_none());
    assert!(index
        .nodes_for_asset("res://ui/icons/save.svg")
        .next()
        .is_none());
}

#[test]
fn surface_index_registers_node_resources_from_template_metadata() {
    let mut tree = UiTree::new(tree_id("runtime.ui.main"));
    tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_template_metadata(
            UiTemplateNodeMetadata {
                attributes: BTreeMap::from([(
                    "icon".to_string(),
                    Value::String("res://ui/icons/run.svg".to_string()),
                )]),
                style_overrides: BTreeMap::from([(
                    "font".to_string(),
                    resource_value(
                        "font",
                        "res://fonts/inter.font.toml",
                        Some(("placeholder", "res://fonts/system.ttf")),
                    ),
                )]),
                ..Default::default()
            },
        ),
    );
    tree.insert_child(
        UiNodeId::new(1),
        UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/image")).with_template_metadata(
            UiTemplateNodeMetadata {
                slot_attributes: BTreeMap::from([(
                    "background_image".to_string(),
                    resource_value(
                        "image",
                        "asset://images/thumbnail.png",
                        Some(("optional", "asset://images/fallback.png")),
                    ),
                )]),
                ..Default::default()
            },
        ),
    )
    .unwrap();
    tree.insert_child(
        UiNodeId::new(1),
        UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/label")),
    )
    .unwrap();

    let mut index = UiAssetSurfaceIndex::new();
    let report = index.record_tree_node_resources(&tree);

    assert_eq!(report.tree_id, tree_id("runtime.ui.main"));
    assert_eq!(report.nodes_registered, 2);
    assert_eq!(report.resource_uris_registered, 4);
    assert_eq!(report.nodes_without_resources, vec![UiNodeId::new(3)]);
    assert_eq!(
        index.assets_for_node(&tree_id("runtime.ui.main"), UiNodeId::new(1)),
        &[
            "res://ui/icons/run.svg".to_string(),
            "res://fonts/inter.font.toml".to_string(),
            "res://fonts/system.ttf".to_string(),
        ]
    );
    assert_eq!(
        index.assets_for_node(&tree_id("runtime.ui.main"), UiNodeId::new(2)),
        &[
            "asset://images/thumbnail.png".to_string(),
            "asset://images/fallback.png".to_string(),
        ]
    );
    assert_eq!(
        index
            .nodes_for_asset("res://fonts/system.ttf")
            .cloned()
            .collect::<Vec<_>>(),
        vec![UiAssetNodeTarget {
            tree_id: tree_id("runtime.ui.main"),
            node_id: UiNodeId::new(1),
        }]
    );
}

#[test]
fn surface_index_tree_resource_registration_removes_stale_node_edges() {
    let mut index = UiAssetSurfaceIndex::new();
    let tree_id_value = tree_id("runtime.ui.main");
    index.record_node_assets(
        tree_id_value.clone(),
        UiNodeId::new(7),
        ["res://ui/icons/old.svg"],
    );

    let mut tree = UiTree::new(tree_id_value.clone());
    tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_template_metadata(
            UiTemplateNodeMetadata {
                attributes: BTreeMap::from([(
                    "icon".to_string(),
                    Value::String("res://ui/icons/new.svg".to_string()),
                )]),
                ..Default::default()
            },
        ),
    );

    let report = index.record_tree_node_resources(&tree);

    assert_eq!(report.nodes_registered, 1);
    assert!(index
        .assets_for_node(&tree_id_value, UiNodeId::new(7))
        .is_empty());
    assert!(index
        .nodes_for_asset("res://ui/icons/old.svg")
        .next()
        .is_none());
    assert_eq!(
        index
            .nodes_for_asset("res://ui/icons/new.svg")
            .cloned()
            .collect::<Vec<_>>(),
        vec![UiAssetNodeTarget {
            tree_id: tree_id_value,
            node_id: UiNodeId::new(1),
        }]
    );
}

#[test]
fn hot_reload_plan_maps_resource_targets_to_precise_nodes_when_registered() {
    let mut dependency_index = UiAssetDependencyIndex::new();
    dependency_index.record_compiled(
        "res://ui/views/main.v2.ui.toml",
        &[
            asset_ref("res://ui/icons/run.svg"),
            asset_ref("res://fonts/inter.font.toml"),
        ],
    );

    let mut surface_index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    let inspector = tree_id("runtime.ui.inspector");
    surface_index.record_surface_assets(
        main.clone(),
        [
            "res://ui/views/main.v2.ui.toml",
            "res://ui/icons/run.svg",
            "res://fonts/inter.font.toml",
        ],
    );
    surface_index.record_surface_assets(
        inspector.clone(),
        [
            "res://ui/views/inspector.v2.ui.toml",
            "res://ui/icons/run.svg",
        ],
    );
    surface_index.record_node_assets(main.clone(), UiNodeId::new(2), ["res://ui/icons/run.svg"]);
    surface_index.record_node_assets(
        main.clone(),
        UiNodeId::new(3),
        ["res://fonts/inter.font.toml"],
    );
    surface_index.record_node_assets(
        inspector.clone(),
        UiNodeId::new(7),
        ["res://ui/icons/run.svg"],
    );

    let report = dependency_index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Modified,
        uri("res://ui/icons/run.svg"),
        None,
    )]);
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);
    let node_targets = surface_index.target_nodes_for_plan(&plan);

    assert_eq!(
        node_targets.resource_damage_nodes,
        vec![
            UiAssetNodeTarget {
                tree_id: inspector,
                node_id: UiNodeId::new(7),
            },
            UiAssetNodeTarget {
                tree_id: main,
                node_id: UiNodeId::new(2),
            },
        ]
    );
    assert!(node_targets.theme_restyle_nodes.is_empty());
    assert!(node_targets.template_rebuild_nodes.is_empty());
}

#[test]
fn hot_reload_plan_marks_target_surface_roots_dirty_and_reports_missing_surfaces() {
    let mut dependency_index = UiAssetDependencyIndex::new();
    dependency_index.record_compiled(
        "res://ui/views/main.v2.ui.toml",
        &[asset_ref("res://fonts/inter.font.toml")],
    );

    let mut surface_index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    let stale = tree_id("runtime.ui.stale");
    surface_index.record_surface_assets(
        main.clone(),
        [
            "res://ui/views/main.v2.ui.toml",
            "res://fonts/inter.font.toml",
        ],
    );
    surface_index.record_surface_assets(stale.clone(), ["res://fonts/inter.font.toml"]);

    let report = dependency_index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Modified,
        uri("res://fonts/inter.font.toml"),
        None,
    )]);
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);
    let mut surfaces = BTreeMap::from([(main.clone(), dirty_test_surface(&main))]);

    let apply = surface_index
        .mark_target_surfaces_dirty(&plan, &mut surfaces)
        .unwrap();

    assert_eq!(
        apply.targets.resource_damage_surfaces,
        vec![main.clone(), stale.clone()]
    );
    assert_eq!(apply.missing_surfaces, vec![stale]);
    let dirty_report = apply.dirty_reports.get(&main).unwrap();
    assert_eq!(dirty_report.roots_marked, 1);
    assert_eq!(
        surfaces.get(&main).unwrap().dirty_flags(),
        UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            text: true,
            ..Default::default()
        }
    );
}

#[test]
fn hot_reload_plan_marks_precise_resource_nodes_and_reports_missing_nodes() {
    let mut dependency_index = UiAssetDependencyIndex::new();
    dependency_index.record_compiled(
        "res://ui/views/main.v2.ui.toml",
        &[asset_ref("res://ui/icons/run.svg")],
    );

    let mut surface_index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    surface_index.record_surface_assets(
        main.clone(),
        ["res://ui/views/main.v2.ui.toml", "res://ui/icons/run.svg"],
    );
    surface_index.record_node_assets(main.clone(), UiNodeId::new(2), ["res://ui/icons/run.svg"]);
    surface_index.record_node_assets(main.clone(), UiNodeId::new(99), ["res://ui/icons/run.svg"]);

    let report = dependency_index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Modified,
        uri("res://ui/icons/run.svg"),
        None,
    )]);
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);
    let mut surfaces = BTreeMap::from([(main.clone(), dirty_test_surface_with_nodes(&main))]);

    let apply = surface_index
        .mark_target_surfaces_dirty(&plan, &mut surfaces)
        .unwrap();

    assert!(apply.dirty_reports.is_empty());
    assert_eq!(
        apply.node_targets.resource_damage_nodes,
        vec![
            UiAssetNodeTarget {
                tree_id: main.clone(),
                node_id: UiNodeId::new(2),
            },
            UiAssetNodeTarget {
                tree_id: main.clone(),
                node_id: UiNodeId::new(99),
            },
        ]
    );
    let report = apply.node_dirty_reports.get(&main).unwrap();
    assert_eq!(report.nodes_marked, vec![UiNodeId::new(2)]);
    assert_eq!(report.missing_nodes, vec![UiNodeId::new(99)]);
    assert_eq!(
        report.dirty,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );

    let surface = surfaces.get(&main).unwrap();
    assert_eq!(
        surface.tree.node(UiNodeId::new(1)).unwrap().dirty,
        UiDirtyFlags::default()
    );
    assert_eq!(
        surface.tree.node(UiNodeId::new(2)).unwrap().dirty,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );
}

#[test]
fn mixed_surface_and_node_targets_fall_back_to_root_dirty() {
    let mut dependency_index = UiAssetDependencyIndex::new();
    dependency_index.record_compiled(
        "res://ui/views/main.v2.ui.toml",
        &[
            asset_ref("res://ui/theme/base.theme.toml"),
            asset_ref("res://ui/icons/run.svg"),
        ],
    );

    let mut surface_index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    surface_index.record_surface_assets(
        main.clone(),
        [
            "res://ui/views/main.v2.ui.toml",
            "res://ui/theme/base.theme.toml",
            "res://ui/icons/run.svg",
        ],
    );
    surface_index.record_node_assets(main.clone(), UiNodeId::new(2), ["res://ui/icons/run.svg"]);

    let report = dependency_index.apply_watch_changes(&[
        AssetChange::new(
            AssetChangeKind::Modified,
            uri("res://ui/theme/base.theme.toml"),
            None,
        ),
        AssetChange::new(
            AssetChangeKind::Modified,
            uri("res://ui/icons/run.svg"),
            None,
        ),
    ]);
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);
    let mut surfaces = BTreeMap::from([(main.clone(), dirty_test_surface_with_nodes(&main))]);

    let apply = surface_index
        .mark_target_surfaces_dirty(&plan, &mut surfaces)
        .unwrap();

    assert!(apply.node_dirty_reports.is_empty());
    assert_eq!(apply.dirty_reports.get(&main).unwrap().roots_marked, 1);
    let surface = surfaces.get(&main).unwrap();
    assert!(surface.tree.node(UiNodeId::new(1)).unwrap().dirty.style);
    assert!(surface.tree.node(UiNodeId::new(1)).unwrap().dirty.render);
    assert_eq!(
        surface.tree.node(UiNodeId::new(2)).unwrap().dirty,
        UiDirtyFlags::default()
    );
}

#[test]
fn template_rebuild_still_uses_surface_level_dirty_even_when_node_edges_exist() {
    let mut dependency_index = UiAssetDependencyIndex::new();
    dependency_index.record_compiled(
        "res://ui/views/main.v2.ui.toml",
        &[asset_ref("res://ui/components/button.zui")],
    );

    let mut surface_index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    surface_index.record_surface_assets(
        main.clone(),
        [
            "res://ui/views/main.v2.ui.toml",
            "res://ui/components/button.zui",
        ],
    );
    surface_index.record_node_assets(
        main.clone(),
        UiNodeId::new(2),
        ["res://ui/components/button.zui"],
    );

    let report = dependency_index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Modified,
        uri("res://ui/components/button.zui"),
        None,
    )]);
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);
    let mut surfaces = BTreeMap::from([(main.clone(), dirty_test_surface_with_nodes(&main))]);

    let apply = surface_index
        .mark_target_surfaces_dirty(&plan, &mut surfaces)
        .unwrap();

    assert!(apply.node_dirty_reports.is_empty());
    assert_eq!(apply.dirty_reports.get(&main).unwrap().roots_marked, 1);
    assert!(surfaces.get(&main).unwrap().dirty_flags().layout);
    assert!(surfaces.get(&main).unwrap().dirty_flags().visible_range);
}

#[test]
fn template_plan_targets_surface_that_owns_compiled_asset() {
    let mut dependency_index = UiAssetDependencyIndex::new();
    dependency_index.record_compiled(
        "res://ui/views/main.v2.ui.toml",
        &[asset_ref("res://ui/components/button.zui")],
    );

    let mut surface_index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    surface_index.record_surface_assets(
        main.clone(),
        [
            "res://ui/views/main.v2.ui.toml",
            "res://ui/components/button.zui",
        ],
    );

    let report = dependency_index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Modified,
        uri("res://ui/components/button.zui"),
        None,
    )]);
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);
    let targets = surface_index.target_surfaces_for_plan(&plan);

    assert_eq!(targets.template_rebuild_surfaces, vec![main]);
    assert!(targets.rebuild_required);
    assert!(targets.dirty.layout);
    assert!(targets.dirty.input);
    assert!(targets.dirty.visible_range);
}

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
