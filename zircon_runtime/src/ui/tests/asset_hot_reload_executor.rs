use std::collections::BTreeMap;

use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetReference, AssetUri};
use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator, ResourceManager};
use crate::ui::surface::UiSurface;
use crate::ui::template::{
    UiAssetCompileCache, UiAssetDependencyIndex, UiAssetHotReloadExecutionError,
    UiAssetHotReloadExecutor, UiAssetHotReloadPlan, UiAssetLoader, UiAssetSurfaceIndex,
    UiBindingReloadPrepareError, UiDocumentCompiler, UiResourceResolver, UiTemplateSurfaceBuilder,
};
use crate::ui::theme::UiThemeRegistry;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use zircon_runtime_interface::ui::layout::{AxisConstraint, BoxConstraints, StretchMode, UiSize};
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiThemeDocument};
use zircon_runtime_interface::ui::template::{
    UiResourceFallbackMode, UiResourceFallbackPolicy, UiResourceKind, UiResourceRef,
};
use zircon_runtime_interface::ui::{
    component::UiValue,
    tree::{UiDirtyFlags, UiTemplateNodeMetadata, UiTreeNode},
};

const COMPILED_TEMPLATE: &str = r#"
[asset]
kind = "layout"
id = "res://ui/views/main.zui"
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

const RELOAD_BINDING_V1: &str = r#"
[asset]
kind = "layout"
id = "res://ui/views/reload-binding.zui"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"

[[root.bindings]]
id = "Root/onActivate"
event = "Click"
route = "reload.v1"
"#;

const RELOAD_BINDING_V2: &str = r#"
[asset]
kind = "layout"
id = "res://ui/views/reload-binding.zui"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"

[[root.bindings]]
id = "Root/onActivate"
event = "Click"
route = "reload.v2"
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
        "res://ui/views/main.zui",
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
            uri("res://ui/views/main.zui"),
            None,
        ),
    ]);
    let plan = UiAssetHotReloadPlan::from_watch_report(&report);

    let tree_id = UiTreeId::new("runtime.ui.main");
    let mut surface_index = UiAssetSurfaceIndex::new();
    surface_index.record_surface_assets(
        tree_id.clone(),
        [
            "res://ui/views/main.zui",
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
                surface_index: &mut surface_index,
                surfaces: &mut surfaces,
                resource_resolver: None,
                theme_registry: Some(&mut theme_registry),
                template_rebuilder: None,
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
        "res://ui/views/main.zui",
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
        ["res://ui/views/main.zui", "res://ui/icons/run.svg"],
    );
    let mut surfaces = BTreeMap::from([(tree_id.clone(), dirty_test_surface(&tree_id))]);
    let mut cache = UiAssetCompileCache::new();

    let execution = plan
        .execute_runtime_reload(
            UiAssetHotReloadExecutor {
                cache: &mut cache,
                surface_index: &mut surface_index,
                surfaces: &mut surfaces,
                resource_resolver: None,
                theme_registry: None,
                template_rebuilder: None,
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
        "res://ui/views/main.zui",
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
            "res://ui/views/main.zui",
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
                surface_index: &mut surface_index,
                surfaces: &mut surfaces,
                resource_resolver: Some(&mut resolver),
                theme_registry: None,
                template_rebuilder: None,
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

#[test]
fn template_hot_reload_prepare_failure_preserves_last_known_good_surfaces_and_cache() {
    let document = UiAssetLoader::load_toml_str(COMPILED_TEMPLATE).unwrap();
    let compiler = UiDocumentCompiler::default();
    let compiled = compiler.compile(&document).unwrap();
    let mut cache = UiAssetCompileCache::new();
    compiler.compile_with_cache(&document, &mut cache).unwrap();

    let asset_id = "res://ui/views/main.zui";
    let first_id = UiTreeId::new("runtime.ui.first");
    let second_id = UiTreeId::new("runtime.ui.second");
    let mut surface_index = UiAssetSurfaceIndex::new();
    let mut surfaces = BTreeMap::from([
        (
            first_id.clone(),
            UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
                first_id.clone(),
                &compiled,
            )
            .unwrap(),
        ),
        (
            second_id.clone(),
            UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
                second_id.clone(),
                &compiled,
            )
            .unwrap(),
        ),
    ]);
    surface_index.record_compiled_surface(first_id.clone(), &compiled);
    surface_index.record_compiled_surface(second_id.clone(), &compiled);
    let last_known_good = surfaces.clone();
    let last_known_good_index = surface_index.clone();
    let plan = template_rebuild_plan(asset_id);
    let mut prepare_calls = 0;
    let mut rebuilder = |request: crate::ui::template::UiAssetSurfaceRebuildRequest<'_>| {
        prepare_calls += 1;
        if request.tree_id == &second_id {
            return Err("synthetic compile failure".to_string());
        }
        UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
            request.tree_id.clone(),
            &compiled,
        )
        .map_err(|error| error.to_string())
    };

    let error = plan
        .execute_runtime_reload(
            UiAssetHotReloadExecutor {
                cache: &mut cache,
                surface_index: &mut surface_index,
                surfaces: &mut surfaces,
                resource_resolver: None,
                theme_registry: None,
                template_rebuilder: Some(&mut rebuilder),
            },
            None,
        )
        .unwrap_err();

    assert!(matches!(
        error,
        UiAssetHotReloadExecutionError::PrepareFailed { tree_id, .. }
            if tree_id == second_id
    ));
    assert_eq!(prepare_calls, 2);
    assert_eq!(surfaces, last_known_good);
    assert_eq!(surface_index, last_known_good_index);
    assert!(!cache.is_empty());
}

#[test]
fn template_hot_reload_atomically_publishes_replacements_and_migrates_compatible_state() {
    let document = UiAssetLoader::load_toml_str(COMPILED_TEMPLATE).unwrap();
    let compiler = UiDocumentCompiler::default();
    let mut cache = UiAssetCompileCache::new();
    compiler.compile_with_cache(&document, &mut cache).unwrap();

    let asset_id = "res://ui/views/main.zui";
    let tree_id = UiTreeId::new("runtime.ui.migrate");
    let mut previous = reload_state_surface(&tree_id, 1, 2, "Label", "Checkbox");
    previous.component_states.set_value(
        UiNodeId::new(1),
        "text",
        UiValue::String("preserved".to_string()),
    );
    previous
        .component_states
        .set_value(UiNodeId::new(2), "checked", UiValue::Bool(true));
    previous
        .component_states
        .set_checked(UiNodeId::new(1), true);
    previous
        .component_states
        .set_focused(UiNodeId::new(1), true);
    previous
        .component_states
        .set_hovered(UiNodeId::new(1), true);
    previous
        .component_states
        .set_pressed(UiNodeId::new(1), true);
    let mut surfaces = BTreeMap::from([(tree_id.clone(), previous)]);
    let mut surface_index = UiAssetSurfaceIndex::new();
    surface_index.record_surface_assets(tree_id.clone(), [asset_id]);
    let plan = template_rebuild_plan(asset_id);
    let mut rebuilder = |request: crate::ui::template::UiAssetSurfaceRebuildRequest<'_>| {
        assert_eq!(request.template_assets, &[asset_id.to_string()]);
        Ok(reload_state_surface(
            request.tree_id,
            10,
            20,
            "Label",
            "Slider",
        ))
    };

    let execution = plan
        .execute_runtime_reload(
            UiAssetHotReloadExecutor {
                cache: &mut cache,
                surface_index: &mut surface_index,
                surfaces: &mut surfaces,
                resource_resolver: None,
                theme_registry: None,
                template_rebuilder: Some(&mut rebuilder),
            },
            None,
        )
        .unwrap();

    assert!(cache.is_empty());
    assert_eq!(execution.template_rebuilds.len(), 1);
    assert_eq!(execution.template_rebuilds[0].component_states_migrated, 1);
    assert_eq!(execution.template_rebuilds[0].component_states_reset, 1);
    assert_eq!(
        execution.template_rebuilds[0]
            .binding_reload
            .state_entries_migrated,
        1
    );
    assert_eq!(
        execution.template_rebuilds[0]
            .binding_reload
            .state_entries_reset,
        1
    );
    let replacement = surfaces.get(&tree_id).unwrap();
    assert!(replacement.tree.node(UiNodeId::new(1)).is_none());
    assert_eq!(
        replacement
            .component_state(UiNodeId::new(10))
            .and_then(|state| state.value("text")),
        Some(&UiValue::String("preserved".to_string()))
    );
    let migrated_state = replacement.component_state(UiNodeId::new(10)).unwrap();
    assert!(migrated_state.flags.checked);
    assert!(!migrated_state.flags.focused);
    assert!(!migrated_state.flags.hovered);
    assert!(!migrated_state.flags.pressed);
    assert!(replacement
        .component_state(UiNodeId::new(20))
        .and_then(|state| state.value("checked"))
        .is_none());
}

#[test]
fn template_hot_reload_migrates_one_thousand_stable_states_in_one_publication() {
    const STATE_COUNT: u64 = 1_000;

    let asset_id = "res://ui/views/large.zui";
    let tree_id = UiTreeId::new("runtime.ui.large");
    let mut previous = UiSurface::new(tree_id.clone());
    let mut replacement = UiSurface::new(tree_id.clone());
    for offset in 0..STATE_COUNT {
        previous.tree.insert_root(reload_state_node(
            offset + 1,
            &format!("Control{offset}"),
            "TextInput",
        ));
        previous.component_states.set_value(
            UiNodeId::new(offset + 1),
            "text",
            UiValue::String(format!("value-{offset}")),
        );
        replacement.tree.insert_root(reload_state_node(
            offset + STATE_COUNT + 1,
            &format!("Control{offset}"),
            "TextInput",
        ));
    }

    let mut surfaces = BTreeMap::from([(tree_id.clone(), previous)]);
    let mut surface_index = UiAssetSurfaceIndex::new();
    surface_index.record_surface_assets(tree_id.clone(), [asset_id]);
    let plan = template_rebuild_plan(asset_id);
    let mut cache = UiAssetCompileCache::new();
    let mut replacement = Some(replacement);
    let mut rebuilder = move |_request: crate::ui::template::UiAssetSurfaceRebuildRequest<'_>| {
        Ok(replacement.take().expect("the surface is prepared once"))
    };

    let execution = plan
        .execute_runtime_reload(
            UiAssetHotReloadExecutor {
                cache: &mut cache,
                surface_index: &mut surface_index,
                surfaces: &mut surfaces,
                resource_resolver: None,
                theme_registry: None,
                template_rebuilder: Some(&mut rebuilder),
            },
            None,
        )
        .unwrap();

    let receipt = &execution.template_rebuilds[0];
    assert_eq!(receipt.component_states_migrated, STATE_COUNT as usize);
    assert_eq!(receipt.component_states_reset, 0);
    assert_eq!(
        receipt.binding_reload.state_entries_migrated,
        STATE_COUNT as usize
    );
    assert_eq!(receipt.binding_reload.state_entries_reset, 0);
    assert_eq!(surfaces.len(), 1);
    assert_eq!(
        surfaces
            .get(&tree_id)
            .unwrap()
            .component_state(UiNodeId::new(STATE_COUNT * 2))
            .and_then(|state| state.value("text")),
        Some(&UiValue::String("value-999".to_string()))
    );
    println!(
        "PERF-RUNTIME74-HOT-RELOAD state_entries={STATE_COUNT} staged_surfaces=1 precommit_publications=0 published_surfaces=1 migrated_states={} reset_states={}",
        receipt.component_states_migrated,
        receipt.component_states_reset,
    );
}

#[test]
fn binding_reload_receipt_proves_generation_publish_and_old_handle_quiescence() {
    let compiler = UiDocumentCompiler::default();
    let previous_document = UiAssetLoader::load_toml_str(RELOAD_BINDING_V1).unwrap();
    let replacement_document = UiAssetLoader::load_toml_str(RELOAD_BINDING_V2).unwrap();
    let previous_compiled = compiler.compile(&previous_document).unwrap();
    let replacement_compiled = compiler.compile(&replacement_document).unwrap();
    let previous_program = previous_compiled.template_instance().binding_program();
    let replacement_program = replacement_compiled.template_instance().binding_program();
    let previous_generation = previous_program.generation();
    let replacement_generation = replacement_program.generation();
    let replacement_handle = replacement_program
        .handle_for_node_binding(
            zircon_runtime_interface::ui::template::UiCompiledNodeId::new(0),
            0,
        )
        .unwrap();
    assert_ne!(previous_generation, replacement_generation);
    let stale_handle = previous_program
        .handle_for_node_binding(
            zircon_runtime_interface::ui::template::UiCompiledNodeId::new(0),
            0,
        )
        .unwrap();

    let tree_id = UiTreeId::new("runtime.ui.binding-reload");
    let previous_surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        tree_id.clone(),
        &previous_compiled,
    )
    .unwrap();
    let replacement_surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        tree_id.clone(),
        &replacement_compiled,
    )
    .unwrap();
    let mut surfaces = BTreeMap::from([(tree_id.clone(), previous_surface)]);
    let mut surface_index = UiAssetSurfaceIndex::new();
    surface_index.record_compiled_surface(tree_id.clone(), &previous_compiled);
    let mut replacement_surface = Some(replacement_surface);
    let mut rebuilder = move |_request: crate::ui::template::UiAssetSurfaceRebuildRequest<'_>| {
        Ok(replacement_surface
            .take()
            .expect("the binding surface is prepared once"))
    };
    let mut cache = UiAssetCompileCache::new();

    let execution = template_rebuild_plan("res://ui/views/reload-binding.zui")
        .execute_runtime_reload(
            UiAssetHotReloadExecutor {
                cache: &mut cache,
                surface_index: &mut surface_index,
                surfaces: &mut surfaces,
                resource_resolver: None,
                theme_registry: None,
                template_rebuilder: Some(&mut rebuilder),
            },
            None,
        )
        .unwrap();

    let receipt = &execution.template_rebuilds[0].binding_reload;
    assert_eq!(receipt.old_generation, previous_generation);
    assert_eq!(receipt.published_generation, replacement_generation);
    assert_eq!(receipt.retired_binding_count, 1);
    assert_eq!(receipt.published_binding_count, 1);
    assert!(receipt.old_generation_quiescent);
    assert!(receipt.stale_handles_rejected);
    let published = surfaces.get(&tree_id).unwrap().binding_program();
    assert_eq!(published.generation(), replacement_generation);
    assert!(published.binding(stale_handle).is_none());
    assert_eq!(
        surface_index
            .bindings_for_asset("res://ui/views/reload-binding.zui")
            .map(|target| target.handle)
            .collect::<Vec<_>>(),
        vec![replacement_handle]
    );
}

#[test]
fn binding_reload_rejects_compiled_to_empty_surface_without_mutation() {
    let compiler = UiDocumentCompiler::default();
    let previous_document = UiAssetLoader::load_toml_str(RELOAD_BINDING_V1).unwrap();
    let previous_compiled = compiler.compile(&previous_document).unwrap();
    let previous_program = previous_compiled.template_instance().binding_program();
    let previous_generation = previous_program.generation();
    let previous_handle = previous_program
        .handle_for_node_binding(
            zircon_runtime_interface::ui::template::UiCompiledNodeId::new(0),
            0,
        )
        .unwrap();

    let tree_id = UiTreeId::new("runtime.ui.binding-reload-empty-replacement");
    let previous_surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        tree_id.clone(),
        &previous_compiled,
    )
    .unwrap();
    let mut surfaces = BTreeMap::from([(tree_id.clone(), previous_surface)]);
    let mut surface_index = UiAssetSurfaceIndex::new();
    surface_index.record_compiled_surface(tree_id.clone(), &previous_compiled);
    let mut replacement = Some(UiSurface::new(tree_id.clone()));
    let mut rebuilder = move |_request: crate::ui::template::UiAssetSurfaceRebuildRequest<'_>| {
        Ok(replacement
            .take()
            .expect("the empty replacement is prepared once"))
    };
    let mut cache = UiAssetCompileCache::new();

    let error = template_rebuild_plan("res://ui/views/reload-binding.zui")
        .execute_runtime_reload(
            UiAssetHotReloadExecutor {
                cache: &mut cache,
                surface_index: &mut surface_index,
                surfaces: &mut surfaces,
                resource_resolver: None,
                theme_registry: None,
                template_rebuilder: Some(&mut rebuilder),
            },
            None,
        )
        .unwrap_err();

    assert!(matches!(
        error,
        UiAssetHotReloadExecutionError::BindingPrepare(
            UiBindingReloadPrepareError::InvalidReplacementGeneration {
                old_generation,
                ..
            }
        ) if old_generation == previous_generation
    ));
    let retained = surfaces.get(&tree_id).unwrap().binding_program();
    assert_eq!(retained.generation(), previous_generation);
    assert!(retained.binding(previous_handle).is_some());
    assert_eq!(
        surface_index
            .bindings_for_asset("res://ui/views/reload-binding.zui")
            .map(|target| target.handle)
            .collect::<Vec<_>>(),
        vec![previous_handle]
    );
}

#[test]
fn binding_reload_same_program_reports_generation_not_retired() {
    let compiler = UiDocumentCompiler::default();
    let document = UiAssetLoader::load_toml_str(RELOAD_BINDING_V1).unwrap();
    let compiled = compiler.compile(&document).unwrap();
    let current_program = compiled.template_instance().binding_program();
    let current_handle = current_program
        .handle_for_node_binding(
            zircon_runtime_interface::ui::template::UiCompiledNodeId::new(0),
            0,
        )
        .unwrap();

    let tree_id = UiTreeId::new("runtime.ui.binding-reload-noop");
    let previous_surface =
        UiTemplateSurfaceBuilder::build_surface_from_compiled_document(tree_id.clone(), &compiled)
            .unwrap();
    let replacement_surface =
        UiTemplateSurfaceBuilder::build_surface_from_compiled_document(tree_id.clone(), &compiled)
            .unwrap();
    let mut surfaces = BTreeMap::from([(tree_id.clone(), previous_surface)]);
    let mut surface_index = UiAssetSurfaceIndex::new();
    surface_index.record_compiled_surface(tree_id.clone(), &compiled);
    let mut replacement = Some(replacement_surface);
    let mut rebuilder = move |_request: crate::ui::template::UiAssetSurfaceRebuildRequest<'_>| {
        Ok(replacement
            .take()
            .expect("the no-op replacement is prepared once"))
    };
    let mut cache = UiAssetCompileCache::new();

    let execution = template_rebuild_plan("res://ui/views/reload-binding.zui")
        .execute_runtime_reload(
            UiAssetHotReloadExecutor {
                cache: &mut cache,
                surface_index: &mut surface_index,
                surfaces: &mut surfaces,
                resource_resolver: None,
                theme_registry: None,
                template_rebuilder: Some(&mut rebuilder),
            },
            None,
        )
        .unwrap();

    let receipt = &execution.template_rebuilds[0].binding_reload;
    assert!(!receipt.old_generation_retired);
    assert!(!receipt.old_generation_quiescent);
    assert!(!receipt.stale_handles_rejected);
    assert!(surfaces
        .get(&tree_id)
        .unwrap()
        .binding_program()
        .binding(current_handle)
        .is_some());
}

fn template_rebuild_plan(asset_id: &str) -> UiAssetHotReloadPlan {
    UiAssetHotReloadPlan {
        changed_assets: vec![asset_id.to_string()],
        template_rebuild_targets: vec![asset_id.to_string()],
        rebuild_required: true,
        dirty: UiDirtyFlags {
            style: true,
            layout: true,
            hit_test: true,
            render: true,
            input: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn reload_state_surface(
    tree_id: &UiTreeId,
    stable_node_id: u64,
    reset_node_id: u64,
    stable_component: &str,
    reset_component: &str,
) -> UiSurface {
    let mut surface = UiSurface::new(tree_id.clone());
    surface.tree.insert_root(reload_state_node(
        stable_node_id,
        "StableControl",
        stable_component,
    ));
    surface.tree.insert_root(reload_state_node(
        reset_node_id,
        "ResetControl",
        reset_component,
    ));
    surface
}

fn reload_state_node(node_id: u64, control_id: &str, component: &str) -> UiTreeNode {
    UiTreeNode::new(
        UiNodeId::new(node_id),
        UiNodePath::new(format!("reload/{control_id}")),
    )
    .with_template_metadata(UiTemplateNodeMetadata {
        component: component.to_string(),
        control_id: Some(control_id.to_string()),
        ..Default::default()
    })
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
