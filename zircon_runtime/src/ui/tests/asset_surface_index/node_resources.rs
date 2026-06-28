use super::*;

#[test]
fn surface_index_tracks_node_asset_edges_and_replaces_stale_node_edges() {
    let mut index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    let run_button = UiNodeId::new(2);
    let status = UiNodeId::new(3);

    index.record_surface_assets(
        main.clone(),
        [
            "res://ui/views/main.zui",
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
        "res://ui/views/main.zui",
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
            "res://ui/views/main.zui",
            "res://ui/icons/run.svg",
            "res://fonts/inter.font.toml",
        ],
    );
    surface_index.record_surface_assets(
        inspector.clone(),
        ["res://ui/views/inspector.zui", "res://ui/icons/run.svg"],
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
