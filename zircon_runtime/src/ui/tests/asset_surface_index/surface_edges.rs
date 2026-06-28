use super::*;

#[test]
fn surface_index_tracks_assets_and_replaces_stale_surface_edges() {
    let mut index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    let inspector = tree_id("runtime.ui.inspector");

    index.record_surface_assets(
        main.clone(),
        [
            "res://ui/views/main.zui",
            "res://ui/theme/base.theme.toml",
            "res://ui/theme/base.theme.toml",
        ],
    );
    index.record_surface_assets(
        inspector.clone(),
        [
            "res://ui/views/inspector.zui",
            "res://ui/theme/base.theme.toml",
        ],
    );

    assert_eq!(index.surface_count(), 2);
    assert_eq!(
        index.assets_for_surface(&main),
        &[
            "res://ui/views/main.zui".to_string(),
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
            "res://ui/views/main.zui".to_string(),
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
        "res://ui/views/main.zui",
        &[
            asset_ref("res://ui/theme/base.theme.toml"),
            asset_ref("res://ui/icons/run.svg"),
        ],
    );
    dependency_index.record_compiled(
        "res://ui/views/inspector.zui",
        &[asset_ref("res://ui/theme/base.theme.toml")],
    );

    let mut surface_index = UiAssetSurfaceIndex::new();
    let main = tree_id("runtime.ui.main");
    let inspector = tree_id("runtime.ui.inspector");
    surface_index.record_surface_assets(
        main.clone(),
        [
            "res://ui/views/main.zui",
            "res://ui/theme/base.theme.toml",
            "res://ui/icons/run.svg",
        ],
    );
    surface_index.record_surface_assets(
        inspector.clone(),
        [
            "res://ui/views/inspector.zui",
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
