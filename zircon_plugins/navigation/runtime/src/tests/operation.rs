use serde_json::json;
use std::time::{Duration, Instant};
use zircon_runtime::asset::{AssetModule, ASSET_MODULE_NAME};
use zircon_runtime::core::framework::navigation::{
    NavMeshBakeRequest, NavigationGeneratedBakeChange, NAVIGATION_BAKE_SCENE_OPERATION,
    NAVIGATION_CLEAR_SURFACE_OPERATION, NAVIGATION_RESTORE_BAKE_OPERATION,
    NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::core::runtime::{CoreRuntime, TasksModule};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::navigation::register_navigation_operation_handlers;
use zircon_runtime::operation::RuntimeOperationService;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::{
    LevelLifecycleState, LevelMetadata, LevelSystem, SceneNavigationRuntime, World,
    WorldPublicationError,
};
use zircon_runtime_interface::{ZrRuntimeOperationSubmitRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use crate::{module_descriptor, navigation_component_descriptors, DefaultNavigationManager};

#[test]
fn runtime_operation_bake_clear_and_restore_owns_real_generated_navmesh_state() {
    let runtime = CoreRuntime::new();
    runtime.register_module(TasksModule.descriptor()).unwrap();
    runtime.register_module(AssetModule.descriptor()).unwrap();
    runtime.activate_module(ASSET_MODULE_NAME).unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    let core = runtime.handle();
    let manager = core
        .resolve_driver::<DefaultNavigationManager>(crate::DEFAULT_NAVIGATION_RUNTIME_DRIVER_NAME)
        .unwrap();
    let mut service = RuntimeOperationService::new();
    register_navigation_operation_handlers(&mut service).unwrap();
    let mut world = World::empty();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let surface = world
        .spawn_node(NodeKind::Empty)
        .expect("operation fixture surface spawn");
    world
        .spawn_node(NodeKind::Cube)
        .expect("operation fixture geometry spawn");
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({"volume_size": [8.0, 4.0, 8.0]}),
        )
        .unwrap();
    let level = zircon_runtime::scene::create_level(&core, world, LevelMetadata::default())
        .expect("operation fixture level creation");

    let baked = run_operation(
        &service,
        &core,
        &level,
        ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            NAVIGATION_BAKE_SCENE_OPERATION,
            serde_json::to_value(NavMeshBakeRequest::default()).unwrap(),
        ),
    );
    let baked: NavigationGeneratedBakeChange = serde_json::from_value(baked).unwrap();
    assert!(baked.before.asset.is_none());
    assert!(baked.after.asset.is_some());
    assert!(baked.report.is_some());
    assert_eq!(baked.after.surface_entity, Some(surface));
    let baked_loaded_assets = manager.loaded_assets();
    assert_eq!(baked_loaded_assets.len(), 1);
    let baked_loaded_handle = baked_loaded_assets[0].0;
    let generated_epoch_after_bake = manager.generated_bake_mutation_epoch(Some(surface));

    let cleared = run_operation(
        &service,
        &core,
        &level,
        ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            NAVIGATION_CLEAR_SURFACE_OPERATION,
            json!({"surface_entity": surface}),
        ),
    );
    let cleared: NavigationGeneratedBakeChange = serde_json::from_value(cleared).unwrap();
    assert!(cleared.before.asset.is_some());
    assert!(cleared.after.asset.is_none());
    assert!(cleared.report.is_none());
    assert!(manager.loaded_assets().is_empty());
    let generated_epoch_after_clear = manager.generated_bake_mutation_epoch(Some(surface));
    assert!(generated_epoch_after_clear > generated_epoch_after_bake);

    let restored = run_operation(
        &service,
        &core,
        &level,
        ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            NAVIGATION_RESTORE_BAKE_OPERATION,
            serde_json::to_value(&cleared.before).unwrap(),
        ),
    );
    let restored: NavigationGeneratedBakeChange = serde_json::from_value(restored).unwrap();
    assert!(restored.before.asset.is_none());
    assert!(restored.after.asset.is_some());
    assert!(restored.report.is_none());
    assert_eq!(manager.loaded_assets().len(), 1);
    let restored_loaded_assets = manager.loaded_assets();
    assert_ne!(
        restored_loaded_assets[0].0, baked_loaded_handle,
        "clear then restore must publish a fresh loaded identity"
    );
    assert!(manager.generated_bake_mutation_epoch(Some(surface)) > generated_epoch_after_clear);
}

#[test]
fn runtime_operation_malformed_bake_is_terminal_without_owner_mutation() {
    let runtime = CoreRuntime::new();
    runtime.register_module(TasksModule.descriptor()).unwrap();
    runtime.register_module(AssetModule.descriptor()).unwrap();
    runtime.activate_module(ASSET_MODULE_NAME).unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    let core = runtime.handle();
    let manager = core
        .resolve_driver::<DefaultNavigationManager>(crate::DEFAULT_NAVIGATION_RUNTIME_DRIVER_NAME)
        .unwrap();
    let mut service = RuntimeOperationService::new();
    register_navigation_operation_handlers(&mut service).unwrap();
    let mut world = World::empty();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let surface = world
        .spawn_node(NodeKind::Empty)
        .expect("malformed operation fixture surface spawn");
    world
        .spawn_node(NodeKind::Cube)
        .expect("malformed operation fixture geometry spawn");
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({"volume_size": [8.0, 4.0, 8.0]}),
        )
        .unwrap();
    let level = zircon_runtime::scene::create_level(&core, world, LevelMetadata::default())
        .expect("malformed operation fixture level creation");
    let baked = run_operation(
        &service,
        &core,
        &level,
        ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            NAVIGATION_BAKE_SCENE_OPERATION,
            serde_json::to_value(NavMeshBakeRequest::default()).unwrap(),
        ),
    );
    let baked: NavigationGeneratedBakeChange = serde_json::from_value(baked).unwrap();
    assert!(baked.after.asset.is_some());
    assert_eq!(manager.loaded_assets().len(), 1);
    let before = manager.generated_bake_snapshot(Some(surface));
    let before_unqualified = manager.generated_bake_snapshot(None);
    let loaded_before = manager.loaded_assets();
    let mutation_epoch_before = manager.generated_bake_mutation_epoch(Some(surface));
    let null_restore = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            NAVIGATION_RESTORE_BAKE_OPERATION,
            json!({
                "surface_entity": null,
                "asset": null,
                "output_asset": null,
            }),
        ))
        .unwrap();
    let null_restore_deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < null_restore_deadline {
        service.tick(&core, &level);
        if service
            .poll(null_restore)
            .unwrap()
            .phase()
            .is_some_and(|phase| phase.is_terminal())
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(service
        .poll(null_restore)
        .unwrap()
        .phase()
        .is_some_and(|phase| phase.is_terminal()));
    let null_restore_result = service.harvest(null_restore).unwrap();
    assert!(null_restore_result.failure().is_some());
    assert_eq!(manager.generated_bake_snapshot(Some(surface)), before);
    assert_eq!(manager.generated_bake_snapshot(None), before_unqualified);
    assert_eq!(manager.loaded_assets(), loaded_before);
    assert_eq!(
        manager.generated_bake_mutation_epoch(Some(surface)),
        mutation_epoch_before,
        "a null restore must not advance the generated-state epoch"
    );
    let handle = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            NAVIGATION_BAKE_SCENE_OPERATION,
            json!({"surface_entity": "wrong-type"}),
        ))
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        service.tick(&core, &level);
        if service
            .poll(handle)
            .unwrap()
            .phase()
            .is_some_and(|phase| phase.is_terminal())
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(service
        .poll(handle)
        .unwrap()
        .phase()
        .is_some_and(|phase| phase.is_terminal()));
    let result = service.harvest(handle).unwrap();
    assert!(result.failure().is_some());
    assert_eq!(manager.generated_bake_snapshot(Some(surface)), before);
    assert_eq!(manager.generated_bake_snapshot(None), before_unqualified);
    assert_eq!(manager.loaded_assets(), loaded_before);
    assert_eq!(
        manager.generated_bake_mutation_epoch(Some(surface)),
        mutation_epoch_before,
        "a wrong-typed bake payload must fail before owner mutation"
    );
}

#[test]
fn runtime_operation_owner_source_rejects_edit_replacement_and_unload_reload_aba() {
    let runtime = CoreRuntime::new();
    runtime.register_module(TasksModule.descriptor()).unwrap();
    runtime.register_module(AssetModule.descriptor()).unwrap();
    runtime.activate_module(ASSET_MODULE_NAME).unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    let core = runtime.handle();
    let level =
        zircon_runtime::scene::create_level(&core, World::empty(), LevelMetadata::default())
            .expect("owner source fixture level creation");

    let edited_source = level.capture();
    level.with_world_mut(|world| {
        world
            .spawn_node(NodeKind::Cube)
            .expect("owner source fixture edit");
    });
    assert!(matches!(
        edited_source.publish(|_| ()),
        Err(WorldPublicationError::WorldGenerationChanged { .. })
    ));

    let replacement_source = level.capture();
    level.replace(World::empty());
    assert!(matches!(
        replacement_source.publish(|_| ()),
        Err(WorldPublicationError::WorldReplacementChanged { .. })
    ));

    let aba_source = level.capture();
    level.set_lifecycle(LevelLifecycleState::Unloaded);
    level.set_lifecycle(LevelLifecycleState::Loaded);
    assert!(matches!(
        aba_source.publish(|_| ()),
        Err(WorldPublicationError::LifecycleChanged { .. })
    ));
    let fresh_source = level.capture();
    assert!(fresh_source.publish(|_| ()).is_ok());
}

fn run_operation(
    service: &RuntimeOperationService,
    core: &zircon_runtime::core::CoreHandle,
    level: &LevelSystem,
    request: ZrRuntimeOperationSubmitRequestV1,
) -> serde_json::Value {
    let handle = service.submit(request).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        service.tick(core, level);
        let status = service.poll(handle).unwrap();
        if status.phase().is_some_and(|phase| phase.is_terminal()) {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(service
        .poll(handle)
        .unwrap()
        .phase()
        .is_some_and(|phase| phase.is_terminal()));
    service
        .harvest(handle)
        .unwrap()
        .succeeded_output()
        .unwrap()
        .clone()
}
