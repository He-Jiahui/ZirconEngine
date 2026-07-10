use zircon_runtime::core::framework::physics::PhysicsWorldStepPlan;
use zircon_runtime::core::manager::resolve_physics_manager;
use zircon_runtime::core::CoreError;
use zircon_runtime::plugin::RuntimeExtensionRegistryError;
use zircon_runtime::scene::ecs::RuntimeSceneSystemContext;
use zircon_runtime::scene::SystemStage;

use crate::manager::apply_synchronized_bodies_to_scene;

#[derive(Clone, Debug, Default)]
pub struct PhysicsRuntimeSystem;

pub const PHYSICS_SYSTEM_SET: &str = "physics.main";
pub const PHYSICS_STEP_SYSTEM: &str = "physics.step";
pub const PHYSICS_SYNC_TO_SCENE_SYSTEM: &str = "physics.sync_to_scene";

pub fn register_runtime_systems(
    module: &mut zircon_plugin_sdk::RuntimePluginModuleRegistration<'_>,
) -> Result<(), RuntimeExtensionRegistryError> {
    module
        .runtime_scene_system(
            PHYSICS_STEP_SYSTEM,
            SystemStage::FixedUpdate,
            run_physics_runtime_system,
        )
        .in_set(PHYSICS_SYSTEM_SET)
        .register()?;
    module
        .runtime_scene_system(
            PHYSICS_SYNC_TO_SCENE_SYSTEM,
            SystemStage::FixedPostUpdate,
            run_physics_sync_to_scene_system,
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

fn run_physics_sync_to_scene_system(
    context: RuntimeSceneSystemContext<'_>,
) -> Result<(), CoreError> {
    let Ok(physics) = resolve_physics_manager(context.core) else {
        return Ok(());
    };
    let Some(sync) = physics.synchronized_world(context.level.world_handle()) else {
        return Ok(());
    };
    context
        .level
        .with_world_mut(|world| apply_synchronized_bodies_to_scene(world, &sync));
    Ok(())
}
