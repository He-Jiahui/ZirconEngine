use serde_json::json;
use zircon_runtime::core::framework::navigation::{
    NavMeshBakeRequest, NavigationGeneratedBakeChange, NAVIGATION_BAKE_SCENE_OPERATION,
    NAVIGATION_CLEAR_SURFACE_OPERATION, NAVIGATION_RESTORE_BAKE_OPERATION,
    NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::core::runtime::CoreRuntime;
use zircon_runtime::navigation::register_navigation_operation_handlers;
use zircon_runtime::operation::{RuntimeOperationContext, RuntimeOperationService};
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::World;
use zircon_runtime_interface::{ZrRuntimeOperationSubmitRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use crate::{module_descriptor, navigation_component_descriptors, DefaultNavigationManager};

#[test]
fn runtime_operation_bake_clear_and_restore_owns_real_generated_navmesh_state() {
    let runtime = CoreRuntime::new();
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
    let surface = world.spawn_node(NodeKind::Empty);
    world.spawn_node(NodeKind::Cube);
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({"volume_size": [8.0, 4.0, 8.0]}),
        )
        .unwrap();

    let baked = run_operation(
        &service,
        &core,
        &mut world,
        ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            NAVIGATION_BAKE_SCENE_OPERATION,
            serde_json::to_value(NavMeshBakeRequest::default()).unwrap(),
        ),
    );
    let baked: NavigationGeneratedBakeChange = serde_json::from_value(baked).unwrap();
    assert!(baked.before.asset.is_none());
    assert!(baked.after.asset.is_some());
    assert_eq!(baked.after.surface_entity, Some(surface));
    assert_eq!(manager.loaded_assets().len(), 1);

    let cleared = run_operation(
        &service,
        &core,
        &mut world,
        ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            NAVIGATION_CLEAR_SURFACE_OPERATION,
            json!({"surface_entity": surface}),
        ),
    );
    let cleared: NavigationGeneratedBakeChange = serde_json::from_value(cleared).unwrap();
    assert!(cleared.before.asset.is_some());
    assert!(cleared.after.asset.is_none());
    assert!(manager.loaded_assets().is_empty());

    let restored = run_operation(
        &service,
        &core,
        &mut world,
        ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            NAVIGATION_RESTORE_BAKE_OPERATION,
            serde_json::to_value(&cleared.before).unwrap(),
        ),
    );
    let restored: NavigationGeneratedBakeChange = serde_json::from_value(restored).unwrap();
    assert!(restored.after.asset.is_some());
    assert_eq!(manager.loaded_assets().len(), 1);
}

fn run_operation(
    service: &RuntimeOperationService,
    core: &zircon_runtime::core::CoreHandle,
    world: &mut World,
    request: ZrRuntimeOperationSubmitRequestV1,
) -> serde_json::Value {
    let handle = service.submit(request).unwrap();
    service
        .poll(RuntimeOperationContext::new(core, world), handle)
        .unwrap();
    service
        .poll(RuntimeOperationContext::new(core, world), handle)
        .unwrap();
    service
        .harvest(handle)
        .unwrap()
        .succeeded_output()
        .unwrap()
        .clone()
}
