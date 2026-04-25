use zircon_runtime::core::framework::physics::{
    PhysicsRayCastQuery, PhysicsSettings, PhysicsSimulationMode, PhysicsWorldStepPlan,
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
mod contact;
mod query;
mod step;
