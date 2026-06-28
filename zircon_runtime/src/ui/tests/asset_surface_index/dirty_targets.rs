use super::*;

#[test]
fn hot_reload_plan_marks_target_surface_roots_dirty_and_reports_missing_surfaces() {
    let mut dependency_index = UiAssetDependencyIndex::new();
    dependency_index.record_compiled(
        "res://ui/views/main.zui",
        &[asset_ref("res://fonts/inter.font.toml")],
    );

    let mut surface_index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    let stale = tree_id("runtime.ui.stale");
    surface_index.record_surface_assets(
        main.clone(),
        ["res://ui/views/main.zui", "res://fonts/inter.font.toml"],
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
        "res://ui/views/main.zui",
        &[asset_ref("res://ui/icons/run.svg")],
    );

    let mut surface_index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    surface_index.record_surface_assets(
        main.clone(),
        ["res://ui/views/main.zui", "res://ui/icons/run.svg"],
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
        "res://ui/views/main.zui",
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
            "res://ui/views/main.zui",
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
        "res://ui/views/main.zui",
        &[asset_ref("res://ui/components/button.zui")],
    );

    let mut surface_index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    surface_index.record_surface_assets(
        main.clone(),
        ["res://ui/views/main.zui", "res://ui/components/button.zui"],
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
        "res://ui/views/main.zui",
        &[asset_ref("res://ui/components/button.zui")],
    );

    let mut surface_index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    surface_index.record_surface_assets(
        main.clone(),
        ["res://ui/views/main.zui", "res://ui/components/button.zui"],
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
