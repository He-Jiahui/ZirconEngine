use std::collections::BTreeMap;

use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetReference, AssetUri};
use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator, ResourceManager};
use crate::ui::surface::UiSurface;
use crate::ui::template::{
    UiAssetCompileCache, UiAssetDependencyIndex, UiAssetHotReloadExecutor, UiAssetHotReloadPlan,
    UiAssetLoader, UiAssetSurfaceIndex, UiDocumentCompiler, UiResourceResolver,
};
use crate::ui::theme::UiThemeRegistry;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use zircon_runtime_interface::ui::layout::{AxisConstraint, BoxConstraints, StretchMode, UiSize};
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiThemeDocument};
use zircon_runtime_interface::ui::template::{
    UiResourceFallbackMode, UiResourceFallbackPolicy, UiResourceKind, UiResourceRef,
};
use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTreeNode};

const COMPILED_TEMPLATE: &str = r#"
[asset]
kind = "layout"
id = "res://ui/views/main.v2.ui.toml"
version = 1

[imports]
resources = [
  { kind = "font", uri = "res://fonts/inter.font.toml", fallback = { mode = "placeholder", uri = "res://fonts/system.ttf" } },
]

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { text = "Main" }
"#;

#[test]
fn executor_evicts_cache_applies_theme_and_marks_registered_surface_dirty() {
    let document = UiAssetLoader::load_toml_str(COMPILED_TEMPLATE).unwrap();
    let compiler = UiDocumentCompiler::default();
    let mut cache = UiAssetCompileCache::new();
    let first = compiler.compile_with_cache(&document, &mut cache).unwrap();
    let second = compiler.compile_with_cache(&document, &mut cache).unwrap();
    assert!(!first.cache_hit);
    assert!(second.cache_hit);

    let mut dependency_index = UiAssetDependencyIndex::new();
    dependency_index.record_compiled(
        "res://ui/views/main.v2.ui.toml",
        &[
            asset_ref("res://ui/theme/base.theme.toml"),
            asset_ref("res://fonts/inter.font.toml"),
        ],
    );
    let report = dependency_index.apply_watch_changes(&[
        AssetChange::new(
            AssetChangeKind::Modified,
            uri("res://ui/theme/base.theme.toml"),
            None,
        ),
        AssetChange::new(
            AssetChangeKind::Modified,
            uri("res://fonts/inter.font.toml"),
            None,
        ),
        AssetChange::new(
            AssetChangeKind::Modified,
            uri("res://ui/views/main.v2.ui.toml"),
            None,
        ),
    ]);
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);

    let tree_id = UiTreeId::new("runtime.ui.main");
    let mut surface_index = UiAssetSurfaceIndex::new();
    surface_index.record_surface_assets(
        tree_id.clone(),
        [
            "res://ui/views/main.v2.ui.toml",
            "res://ui/theme/base.theme.toml",
            "res://fonts/inter.font.toml",
        ],
    );
    let mut surfaces = BTreeMap::from([(tree_id.clone(), dirty_test_surface(&tree_id))]);
    let mut theme_registry = UiThemeRegistry::default();
    let next_theme = changed_theme_document();

    let execution = plan
        .execute_runtime_reload(
            UiAssetHotReloadExecutor {
                cache: &mut cache,
                surface_index: &surface_index,
                surfaces: &mut surfaces,
                resource_resolver: None,
                theme_registry: Some(&mut theme_registry),
            },
            Some(next_theme.clone()),
        )
        .unwrap();

    assert!(cache.is_empty());
    assert_eq!(execution.cache_eviction.entries_removed, 1);
    assert_eq!(execution.cache_eviction.snapshots_removed, 1);
    assert_eq!(
        execution
            .theme_reload
            .as_ref()
            .map(|outcome| outcome.changed),
        Some(true)
    );
    assert_eq!(theme_registry.active().id, next_theme.id);
    assert_eq!(
        execution.resource_refresh_assets,
        vec!["res://fonts/inter.font.toml"]
    );
    assert_eq!(
        execution.surface_apply.targets.theme_restyle_surfaces,
        vec![tree_id.clone()]
    );
    assert_eq!(
        execution.surface_apply.targets.resource_damage_surfaces,
        vec![tree_id.clone()]
    );
    assert_eq!(
        surfaces.get(&tree_id).unwrap().dirty_flags(),
        UiDirtyFlags {
            style: true,
            layout: true,
            hit_test: true,
            render: true,
            text: true,
            input: true,
            visible_range: true,
            ..Default::default()
        }
    );
}

#[test]
fn executor_reports_resource_refresh_without_theme_reload_when_no_theme_document_is_supplied() {
    let mut dependency_index = UiAssetDependencyIndex::new();
    dependency_index.record_compiled(
        "res://ui/views/main.v2.ui.toml",
        &[asset_ref("res://ui/icons/run.svg")],
    );
    let report = dependency_index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Modified,
        uri("res://ui/icons/run.svg"),
        None,
    )]);
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);

    let tree_id = UiTreeId::new("runtime.ui.main");
    let mut surface_index = UiAssetSurfaceIndex::new();
    surface_index.record_surface_assets(
        tree_id.clone(),
        ["res://ui/views/main.v2.ui.toml", "res://ui/icons/run.svg"],
    );
    let mut surfaces = BTreeMap::from([(tree_id.clone(), dirty_test_surface(&tree_id))]);
    let mut cache = UiAssetCompileCache::new();

    let execution = plan
        .execute_runtime_reload(
            UiAssetHotReloadExecutor {
                cache: &mut cache,
                surface_index: &surface_index,
                surfaces: &mut surfaces,
                resource_resolver: None,
                theme_registry: None,
            },
            None,
        )
        .unwrap();

    assert_eq!(execution.cache_eviction.entries_removed, 0);
    assert!(execution.theme_reload.is_none());
    assert_eq!(
        execution.resource_refresh_assets,
        vec!["res://ui/icons/run.svg"]
    );
    assert_eq!(
        surfaces.get(&tree_id).unwrap().dirty_flags(),
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );
}

#[test]
fn executor_invalidates_resource_resolver_cache_for_refreshed_resources() {
    let icon_locator = locator("res://ui/icons/run.svg");
    let icon_id = ResourceId::from_locator(&icon_locator);
    let fallback_locator = locator("res://ui/icons/fallback.svg");
    let fallback_id = ResourceId::from_locator(&fallback_locator);
    let manager = ResourceManager::new();
    manager.register_record(crate::core::resource::ResourceRecord::new(
        icon_id,
        ResourceKind::Texture,
        icon_locator.clone(),
    ));
    manager.register_record(crate::core::resource::ResourceRecord::new(
        fallback_id,
        ResourceKind::Texture,
        fallback_locator.clone(),
    ));
    let mut resolver = UiResourceResolver::new(manager);
    let icon_ref = resource_ref(UiResourceKind::Image, icon_locator.to_string());
    let fallback_ref = UiResourceRef {
        kind: UiResourceKind::Image,
        uri: "res://ui/icons/missing.svg".to_string(),
        fallback: UiResourceFallbackPolicy {
            mode: UiResourceFallbackMode::Placeholder,
            uri: Some(fallback_locator.to_string()),
        },
    };
    resolver.resolve(&icon_ref);
    resolver.resolve(&fallback_ref);
    assert_eq!(resolver.cache_len(), 2);

    let mut dependency_index = UiAssetDependencyIndex::new();
    dependency_index.record_compiled(
        "res://ui/views/main.v2.ui.toml",
        &[
            asset_ref("res://ui/icons/run.svg"),
            asset_ref("res://ui/icons/fallback.svg"),
        ],
    );
    let report = dependency_index.apply_watch_changes(&[AssetChange::new(
        AssetChangeKind::Modified,
        uri("res://ui/icons/fallback.svg"),
        None,
    )]);
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);
    let tree_id = UiTreeId::new("runtime.ui.main");
    let mut surface_index = UiAssetSurfaceIndex::new();
    surface_index.record_surface_assets(
        tree_id.clone(),
        [
            "res://ui/views/main.v2.ui.toml",
            "res://ui/icons/run.svg",
            "res://ui/icons/fallback.svg",
        ],
    );
    let mut surfaces = BTreeMap::from([(tree_id.clone(), dirty_test_surface(&tree_id))]);
    let mut cache = UiAssetCompileCache::new();

    let execution = plan
        .execute_runtime_reload(
            UiAssetHotReloadExecutor {
                cache: &mut cache,
                surface_index: &surface_index,
                surfaces: &mut surfaces,
                resource_resolver: Some(&mut resolver),
                theme_registry: None,
            },
            None,
        )
        .unwrap();

    let invalidation = execution.resource_resolver_cache.unwrap();
    assert_eq!(
        invalidation.requested_uris,
        vec!["res://ui/icons/fallback.svg"]
    );
    assert_eq!(invalidation.references_removed, 1);
    assert_eq!(resolver.cache_len(), 1);
    assert_eq!(resolver.resolve(&icon_ref), resolver.resolve(&icon_ref));
}

fn asset_ref(value: &str) -> AssetReference {
    AssetReference::from_locator(uri(value))
}

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

fn locator(value: &str) -> ResourceLocator {
    ResourceLocator::parse(value).unwrap()
}

fn resource_ref(kind: UiResourceKind, uri: String) -> UiResourceRef {
    UiResourceRef {
        kind,
        uri,
        fallback: UiResourceFallbackPolicy::default(),
    }
}

fn changed_theme_document() -> UiThemeDocument {
    let mut theme = UiThemeDocument::dark();
    theme.id = "zircon.dark.hot_reload".to_string();
    theme.palette.accent = UiRgbaColor::from_u8(80, 210, 190, 255);
    theme
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
