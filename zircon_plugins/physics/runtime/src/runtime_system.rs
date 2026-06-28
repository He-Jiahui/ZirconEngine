use zircon_runtime::core::framework::physics::PhysicsWorldStepPlan;
use zircon_runtime::core::manager::resolve_physics_manager;
use zircon_runtime::core::CoreError;
use zircon_runtime::plugin::RuntimeExtensionRegistryError;
use zircon_runtime::scene::ecs::RuntimeSceneSystemContext;
use zircon_runtime::scene::SystemStage;

#[derive(Clone, Debug, Default)]
pub struct PhysicsRuntimeSystem;

pub const PHYSICS_SYSTEM_SET: &str = "physics.simulation";
pub const PHYSICS_STEP_SYSTEM: &str = "physics.step";

pub fn register_runtime_system(
    module: &mut zircon_plugin_sdk::RuntimePluginModuleRegistration<'_>,
) -> Result<(), RuntimeExtensionRegistryError> {
    module
        .runtime_scene_system(
            PHYSICS_STEP_SYSTEM,
            SystemStage::FixedUpdate,
            run_physics_runtime_system,
        )
        .in_set(PHYSICS_SYSTEM_SET)
        .register()
}

fn run_physics_runtime_system(context: RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> {
    let Ok(physics) = resolve_physics_manager(context.core) else {
        context
            .level
            .record_physics_step(PhysicsWorldStepPlan::default(), Vec::new(), Vec::new());
        return Ok(());
    };

    let result = context.level.with_world_mut(|world| {
        physics.tick_scene_world(context.level.world_handle(), world, context.delta_seconds)
    });
    context
        .level
        .record_physics_step(result.step_plan, result.contacts, result.triggers);
    Ok(())
}
