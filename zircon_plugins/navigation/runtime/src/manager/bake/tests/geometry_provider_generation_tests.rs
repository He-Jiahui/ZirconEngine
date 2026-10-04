use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::{AssetManager, AssetUri, ProjectManifest, ProjectPaths};
use zircon_runtime::core::framework::navigation::{
    NavMeshAsset, NavMeshBakeDiagnostic, NavMeshBakeRequest, NavigationGeneratedBakeSnapshot,
    NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::scene::{components::NodeKind, SceneNavigationRuntime, World};

use crate::manager::NavMeshDirtyBounds;
use crate::navigation_component_descriptors;
use crate::test_support::{level_from_world, navigation_manager};

use super::{ensure_world_generation, publish_bake_with_source_fence};

#[test]
fn stale_project_asset_generation_cannot_publish_a_navigation_bake() {
    let manager = navigation_manager();
    let root = std::env::temp_dir().join(format!(
        "zircon-nav-generation-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the unix epoch")
            .as_nanos()
    ));
    let paths = ProjectPaths::from_root(&root).expect("temporary project paths");
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .expect("temporary project layout");
    ProjectManifest::new(
        "Navigation Generation Fence",
        AssetUri::parse("res://scenes/main.scene.toml").expect("default scene locator"),
        1,
    )
    .save(paths.manifest_path())
    .expect("temporary project manifest");
    let project_assets = manager
        .project_asset_access()
        .resolve()
        .expect("test ProjectAssetManager should resolve");
    AssetManager::open_project(project_assets.as_ref(), root.to_string_lossy().as_ref())
        .expect("temporary project should open");
    let project_asset_generation = project_assets
        .current_project_generation_snapshot()
        .expect("open project should expose a generation snapshot")
        .into_parts()
        .1;
    AssetManager::close_project(project_assets.as_ref()).expect("project should close");

    let (_level_owner, level) = level_from_world(World::empty());
    let source = level.capture();
    let generated_mutation_epoch = manager.capture_generated_mutation_epoch();
    let before_snapshot = manager.generated_bake_snapshot(None);
    let before_loaded_assets = manager.loaded_assets();
    let generation = manager
        .begin_bake_generation(None)
        .expect("generation admission");
    let fresh_snapshot = NavigationGeneratedBakeSnapshot {
        surface_entity: None,
        asset: Some(NavMeshAsset::simple_quad("humanoid", 8.0)),
        output_asset: Some("res://navigation/generated/project-reloaded.navmesh".to_owned()),
    };
    let result = publish_bake_with_source_fence(
        &manager,
        &source,
        source.generation(),
        Some(&project_asset_generation),
        None,
        generation,
        generated_mutation_epoch,
        None,
        fresh_snapshot.clone(),
        Vec::<NavMeshBakeDiagnostic>::new(),
        (0, 0, 0),
    );

    let error = result.expect_err("a closed project's bake snapshot must be rejected");
    assert!(error
        .to_string()
        .contains("project asset generation changed"));
    assert_eq!(manager.generated_bake_snapshot(None), before_snapshot);
    assert_eq!(manager.loaded_assets(), before_loaded_assets);
    assert_eq!(
        manager.capture_generated_mutation_epoch(),
        generated_mutation_epoch
    );
    AssetManager::open_project(project_assets.as_ref(), root.to_string_lossy().as_ref())
        .expect("project should reopen for a fresh generation");
    let fresh_project_asset_generation = project_assets
        .current_project_generation_snapshot()
        .expect("reopened project should expose a fresh generation snapshot")
        .into_parts()
        .1;
    let fresh_generation = manager
        .begin_bake_generation(None)
        .expect("fresh bake generation should be admitted");
    publish_bake_with_source_fence(
        &manager,
        &source,
        source.generation(),
        Some(&fresh_project_asset_generation),
        None,
        fresh_generation,
        manager.capture_generated_mutation_epoch(),
        None,
        fresh_snapshot.clone(),
        Vec::<NavMeshBakeDiagnostic>::new(),
        (0, 0, 0),
    )
    .expect("fresh project generation should publish");
    assert_eq!(manager.generated_bake_snapshot(None), fresh_snapshot);
    let loaded = manager.loaded_assets();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].1.as_ref(), fresh_snapshot.asset.as_ref().unwrap());
    let first_loaded_handle = loaded[0].0;
    manager
        .replace_generated_bake_snapshot(NavigationGeneratedBakeSnapshot::empty(None))
        .expect("A-to-empty transition must release the loaded identity");
    assert!(manager.loaded_assets().is_empty());
    AssetManager::close_project(project_assets.as_ref()).expect("reopened project should close");
    AssetManager::open_project(project_assets.as_ref(), root.to_string_lossy().as_ref())
        .expect("project should reopen after the empty transition");
    let project_asset_generation_after_empty = project_assets
        .current_project_generation_snapshot()
        .expect("second reopened project should expose a generation snapshot")
        .into_parts()
        .1;
    let generation_after_empty = manager
        .begin_bake_generation(None)
        .expect("generation after an empty transition should be admitted");
    publish_bake_with_source_fence(
        &manager,
        &source,
        source.generation(),
        Some(&project_asset_generation_after_empty),
        None,
        generation_after_empty,
        manager.capture_generated_mutation_epoch(),
        None,
        fresh_snapshot.clone(),
        Vec::<NavMeshBakeDiagnostic>::new(),
        (0, 0, 0),
    )
    .expect("fresh project generation should publish after A to empty to A");
    assert_eq!(manager.generated_bake_snapshot(None), fresh_snapshot);
    let reloaded = manager.loaded_assets();
    assert_eq!(reloaded.len(), 1);
    assert_ne!(reloaded[0].0, first_loaded_handle);
    assert_eq!(
        reloaded[0].1.as_ref(),
        fresh_snapshot.asset.as_ref().unwrap()
    );
    AssetManager::close_project(project_assets.as_ref())
        .expect("second reopened project should close");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn world_generation_fence_rejects_a_changed_source_world() {
    let mut world = World::empty();
    let captured = world.world_generation();
    world
        .spawn_node(zircon_runtime::scene::components::NodeKind::Cube)
        .unwrap();

    let error = ensure_world_generation(&world, captured)
        .expect_err("geometry prepared from an earlier World revision must be rejected");

    assert!(error.to_string().contains("World generation"));
}

#[test]
fn max_generation_admission_is_failure_atomic_for_direct_tiled_dirty_and_operation_routes() {
    let manager = navigation_manager();
    let mut world = World::empty();
    for descriptor in navigation_component_descriptors() {
        world
            .register_component_type(descriptor)
            .expect("generation boundary fixture component registration");
    }
    let surface = world
        .spawn_node(NodeKind::Empty)
        .expect("generation boundary fixture surface");
    world
        .spawn_node(NodeKind::Cube)
        .expect("generation boundary fixture geometry");
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            serde_json::json!({"volume_size": [8.0, 4.0, 8.0]}),
        )
        .expect("generation boundary fixture surface settings");
    let (_level_owner, level) = level_from_world(world);
    {
        let mut state = manager.lock_state();
        state.bake_contexts.insert(
            Some(surface),
            super::super::state::BakeContextState {
                next_generation: u64::MAX,
                current_generation: u64::MAX - 1,
                last_tiled_bake: None,
            },
        );
    }
    let before_context = manager
        .lock_state()
        .bake_contexts
        .get(&Some(surface))
        .map(|context| (context.current_generation, context.next_generation));
    let direct_error = manager
        .bake_surface(
            &level,
            NavMeshBakeRequest {
                surface_entity: Some(surface),
                ..NavMeshBakeRequest::default()
            },
        )
        .expect_err("direct/full admission must reject an exhausted generation");
    assert!(direct_error.to_string().contains("generation exhausted"));

    let tiled_handle = manager.start_tiled_bake(
        &level,
        NavMeshBakeRequest {
            surface_entity: Some(surface),
            ..NavMeshBakeRequest::default()
        },
    );
    let tiled_error = manager
        .try_harvest_tiled_bake(tiled_handle)
        .expect("exhausted tiled admission is terminal")
        .expect_err("exhausted tiled admission must fail");
    assert!(tiled_error.to_string().contains("generation exhausted"));

    let dirty_handle = manager.start_dirty_tile_rebuild(
        &level,
        NavMeshBakeRequest {
            surface_entity: Some(surface),
            ..NavMeshBakeRequest::default()
        },
        NavMeshDirtyBounds::new([-1.0; 3], [1.0; 3]),
    );
    let dirty_error = manager
        .try_harvest_dirty_tile_rebuild(dirty_handle)
        .expect("exhausted dirty admission is terminal")
        .expect_err("exhausted dirty admission must fail");
    assert!(dirty_error.to_string().contains("generation exhausted"));

    let token = manager.capture_bake_generation(Some(surface));
    let before_snapshot = manager.generated_bake_snapshot(Some(surface));
    let operation_error = manager
        .publish_operation_bake(
            token,
            &before_snapshot,
            None,
            NavigationGeneratedBakeSnapshot::empty(Some(surface)),
            Vec::new(),
            (0, 0, 0),
        )
        .expect_err("exhausted operation publication must fail");
    assert!(operation_error.to_string().contains("generation exhausted"));
    assert_eq!(
        manager
            .lock_state()
            .bake_contexts
            .get(&Some(surface))
            .map(|context| (context.current_generation, context.next_generation)),
        before_context,
        "every exhausted route must leave its generation context unchanged"
    );
    assert_eq!(
        manager.generated_bake_snapshot(Some(surface)),
        before_snapshot
    );
}
