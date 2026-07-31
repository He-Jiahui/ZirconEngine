use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetReference, AssetUri};
use crate::ui::surface::UiSurface;
use crate::ui::template::{
    classify_ui_hot_reload_asset, UiAssetCompileCache, UiAssetDependencyIndex,
    UiAssetHotReloadPlan, UiAssetLoader, UiDocumentCompiler, UiHotReloadAssetKind,
};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use zircon_runtime_interface::ui::layout::{AxisConstraint, BoxConstraints, StretchMode, UiSize};
use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTreeNode};

const COMPILED_TEMPLATE: &str = r#"
[asset]
kind = "layout"
id = "res://ui/views/main.zui"
version = 1

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { text = "Main" }
"#;

#[test]
fn theme_watch_report_routes_to_restyle_without_template_rebuild() {
    let report = report_for_change(
        "res://ui/theme/editor.theme.toml",
        &[(
            "res://ui/views/main.zui",
            "res://ui/theme/editor.theme.toml",
        )],
    );

    let plan = UiAssetHotReloadPlan::from_watch_report(&report);

    assert_eq!(
        plan.theme_restyle_assets,
        vec!["res://ui/theme/editor.theme.toml"]
    );
    assert_eq!(plan.theme_restyle_targets, vec!["res://ui/views/main.zui"]);
    assert!(plan.template_rebuild_targets.is_empty());
    assert!(!plan.rebuild_required);
    assert!(plan.dirty.style);
    assert!(plan.dirty.layout);
    assert!(plan.dirty.hit_test);
    assert!(plan.dirty.render);
    assert!(plan.dirty.text);
    assert!(!plan.dirty.input);
    assert!(!plan.dirty.visible_range);
}

#[test]
fn icon_watch_report_routes_to_resource_damage_without_rebuild() {
    let report = report_for_change(
        "res://ui/icons/run.icon.toml",
        &[("res://ui/views/main.zui", "res://ui/icons/run.icon.toml")],
    );

    let plan = UiAssetHotReloadPlan::from_watch_report(&report);

    assert_eq!(
        plan.resource_refresh_assets,
        vec!["res://ui/icons/run.icon.toml"]
    );
    assert_eq!(
        plan.resource_damage_targets,
        vec!["res://ui/views/main.zui"]
    );
    assert!(plan.template_rebuild_targets.is_empty());
    assert!(plan.theme_restyle_targets.is_empty());
    assert!(!plan.rebuild_required);
    assert!(plan.dirty.render);
    assert!(!plan.dirty.layout);
    assert!(!plan.dirty.text);
}

#[test]
fn font_watch_report_marks_referencing_targets_for_text_layout_and_render() {
    let report = report_for_change(
        "res://fonts/inter.font.toml",
        &[("res://ui/views/main.zui", "res://fonts/inter.font.toml")],
    );

    let plan = UiAssetHotReloadPlan::from_watch_report(&report);

    assert_eq!(
        plan.resource_refresh_assets,
        vec!["res://fonts/inter.font.toml"]
    );
    assert_eq!(
        plan.resource_damage_targets,
        vec!["res://ui/views/main.zui"]
    );
    assert!(plan.dirty.text);
    assert!(plan.dirty.layout);
    assert!(plan.dirty.hit_test);
    assert!(plan.dirty.render);
    assert!(!plan.dirty.style);
    assert!(!plan.rebuild_required);
}

#[test]
fn template_watch_report_rebuilds_changed_template_and_transitive_dependents() {
    let mut index = UiAssetDependencyIndex::new();
    index.record_compiled(
        "res://ui/views/main.zui",
        &[asset_ref("res://ui/components/button.zui")],
    );

    let report = index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Modified,
        uri("res://ui/components/button.zui"),
        None,
    )]);
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);

    assert_eq!(
        plan.template_rebuild_targets,
        vec!["res://ui/components/button.zui", "res://ui/views/main.zui",]
    );
    assert!(plan.rebuild_required);
    assert!(plan.dirty.layout);
    assert!(plan.dirty.hit_test);
    assert!(plan.dirty.render);
    assert!(plan.dirty.style);
    assert!(plan.dirty.text);
    assert!(plan.dirty.input);
    assert!(plan.dirty.visible_range);
}

#[test]
fn removed_template_evicts_compiled_asset_and_rebuilds_dependents() {
    let mut index = UiAssetDependencyIndex::new();
    index.record_compiled(
        "res://ui/components/button.zui",
        &[asset_ref("res://ui/theme/editor.theme.toml")],
    );
    index.record_compiled(
        "res://ui/views/main.zui",
        &[asset_ref("res://ui/components/button.zui")],
    );

    let report = index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Removed,
        uri("res://ui/components/button.zui"),
        None,
    )]);
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);

    assert_eq!(
        plan.removed_compiled_assets,
        vec!["res://ui/components/button.zui"]
    );
    assert_eq!(
        plan.template_rebuild_targets,
        vec!["res://ui/views/main.zui"]
    );
    assert!(plan.rebuild_required);
}

#[test]
fn texture_source_watch_report_routes_to_resource_damage() {
    let report = report_for_change(
        "res://ui/textures/checker.png",
        &[("res://ui/views/main.zui", "res://ui/textures/checker.png")],
    );

    let plan = UiAssetHotReloadPlan::from_watch_report(&report);

    assert_eq!(
        plan.resource_refresh_assets,
        vec!["res://ui/textures/checker.png"]
    );
    assert_eq!(
        plan.resource_damage_targets,
        vec!["res://ui/views/main.zui"]
    );
    assert!(plan.dirty.render);
    assert!(!plan.rebuild_required);
}

#[test]
fn classifier_matches_ui_asset_suffixes_and_keeps_unknown_visible() {
    assert_eq!(
        classify_ui_hot_reload_asset("res://ui/theme/editor.theme.toml"),
        UiHotReloadAssetKind::Theme
    );
    assert_eq!(
        classify_ui_hot_reload_asset("res://ui/icons/run.icon.toml"),
        UiHotReloadAssetKind::Icon
    );
    assert_eq!(
        classify_ui_hot_reload_asset("res://ui/icons/run.svg"),
        UiHotReloadAssetKind::Icon
    );
    assert_eq!(
        classify_ui_hot_reload_asset("res://fonts/inter.font.toml"),
        UiHotReloadAssetKind::Font
    );
    assert_eq!(
        classify_ui_hot_reload_asset("res://fonts/inter.ttf"),
        UiHotReloadAssetKind::Font
    );
    assert_eq!(
        classify_ui_hot_reload_asset("res://ui/components/button.zui#Button"),
        UiHotReloadAssetKind::Template
    );
    assert_eq!(
        classify_ui_hot_reload_asset("res://ui/views/main.zui"),
        UiHotReloadAssetKind::Template
    );
    assert_eq!(
        classify_ui_hot_reload_asset("res://ui/views/main.v2.ui.toml"),
        UiHotReloadAssetKind::Other
    );
    assert_eq!(
        classify_ui_hot_reload_asset("res://ui/views/main.ui.toml"),
        UiHotReloadAssetKind::Other
    );
    assert_eq!(
        classify_ui_hot_reload_asset("res://ui/textures/checker.ktx2"),
        UiHotReloadAssetKind::Texture
    );
    assert_eq!(
        classify_ui_hot_reload_asset("res://data/unknown.asset"),
        UiHotReloadAssetKind::Other
    );
}

#[test]
fn hot_reload_plan_evicts_compiled_template_cache_entries() {
    let document = UiAssetLoader::load_toml_str(COMPILED_TEMPLATE).unwrap();
    let compiler = UiDocumentCompiler::default();
    let mut cache = UiAssetCompileCache::new();
    let mut index = UiAssetDependencyIndex::new();
    index.record_compiled(
        "res://ui/views/main.zui",
        &[asset_ref("res://ui/components/button.zui")],
    );

    let first = compiler.compile_with_cache(&document, &mut cache).unwrap();
    let second = compiler.compile_with_cache(&document, &mut cache).unwrap();
    assert!(!first.cache_hit);
    assert!(second.cache_hit);
    assert_eq!(cache.len(), 1);

    let report = index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Modified,
        uri("res://ui/components/button.zui"),
        None,
    )]);
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);
    let eviction = plan.evict_compile_cache(&mut cache);

    assert_eq!(eviction.entries_removed, 1);
    assert_eq!(eviction.snapshots_removed, 1);
    assert!(cache.is_empty());

    let third = compiler.compile_with_cache(&document, &mut cache).unwrap();
    assert!(!third.cache_hit);
}

#[test]
fn hot_reload_plan_marks_surface_roots_dirty_with_aggregate_dirty_domains() {
    let report = report_for_change(
        "res://fonts/inter.font.toml",
        &[("res://ui/views/main.zui", "res://fonts/inter.font.toml")],
    );
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);
    let mut surface = dirty_test_surface();

    let dirty = plan.mark_surface_roots_dirty(&mut surface).unwrap();

    assert_eq!(dirty.roots_marked, 1);
    assert_eq!(
        dirty.dirty,
        UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            text: true,
            ..Default::default()
        }
    );
    assert_eq!(surface.dirty_flags(), dirty.dirty);
}

#[test]
fn hot_reload_plan_rebuild_dirty_surface_consumes_planned_dirty_domains() {
    let report = report_for_change(
        "res://ui/icons/run.icon.toml",
        &[("res://ui/views/main.zui", "res://ui/icons/run.icon.toml")],
    );
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);
    let mut surface = dirty_test_surface();

    let rebuild = plan
        .rebuild_dirty_surface(&mut surface, UiSize::new(120.0, 60.0))
        .unwrap();

    assert_eq!(
        rebuild.dirty_flags,
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );
    assert!(rebuild.render_rebuilt);
    assert!(!rebuild.layout_recomputed);
    assert!(!surface.dirty_flags().any());
}

#[test]
fn hot_reload_classifier_borrows_normalized_lowercase_asset_paths() {
    assert_eq!(
        classify_ui_hot_reload_asset(" RES://UI/VIEWS/MAIN.ZUI#Root "),
        UiHotReloadAssetKind::Template
    );

    let source = include_str!("../template/asset/hot_reload_plan.rs");
    assert!(source.contains("Cow::Borrowed(path)"));
    assert!(source.contains("Cow::Owned(path.to_ascii_lowercase())"));
}

fn report_for_change(
    changed: &str,
    compiled_edges: &[(&str, &str)],
) -> crate::ui::template::UiAssetWatchInvalidationReport {
    let mut index = UiAssetDependencyIndex::new();
    for (asset, dependency) in compiled_edges {
        index.record_compiled(asset, &[asset_ref(dependency)]);
    }
    index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Modified,
        uri(changed),
        None,
    )])
}

fn asset_ref(value: &str) -> AssetReference {
    AssetReference::from_locator(uri(value))
}

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

fn dirty_test_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.hot_reload.plan"));
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
