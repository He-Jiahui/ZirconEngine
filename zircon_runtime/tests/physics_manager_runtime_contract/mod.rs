use zircon_runtime::core::framework::physics::{
    PhysicsBackendState, PhysicsRayCastQuery, PhysicsSettings, PhysicsSimulationMode,
    PhysicsWorldStepPlan,
};
use zircon_runtime::core::manager::resolve_physics_manager;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::FOUNDATION_MODULE_NAME;
use zircon_runtime::physics::integrate_builtin_physics_steps;
use zircon_runtime::scene::components::{
    ColliderComponent, ColliderShape, NodeKind, RigidBodyComponent, RigidBodyType,
};
use zircon_runtime::scene::{create_default_level, SCENE_MODULE_NAME};

fn create_runtime_with_scene_and_physics() -> CoreRuntime {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(zircon_runtime::foundation::module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::scene::module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::physics::module_descriptor())
        .unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    runtime
        .activate_module(zircon_runtime::physics::PHYSICS_MODULE_NAME)
        .unwrap();
    runtime
}

#[test]
fn empty_jolt_feature_slot_reports_unavailable_not_ready() {
    let runtime = create_runtime_with_scene_and_physics();
    runtime
        .resolve_manager::<zircon_runtime::physics::DefaultPhysicsManager>(
            "PhysicsModule.Manager.DefaultPhysicsManager",
        )
        .unwrap()
        .store_settings(PhysicsSettings {
            backend: "jolt".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            ..PhysicsSettings::default()
        })
        .unwrap();

    let status = resolve_physics_manager(&runtime.handle())
        .unwrap()
        .backend_status();

    assert_eq!(status.requested_backend, "jolt");
    assert_eq!(status.active_backend, None);
    assert_eq!(status.state, PhysicsBackendState::Unavailable);
    assert_eq!(status.feature_gate.as_deref(), Some("jolt"));
}

mod contact;
mod query;
mod step;
