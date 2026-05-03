use zircon_runtime::core::framework::physics::PhysicsWorldStepPlan;
use zircon_runtime::core::manager::resolve_physics_manager;
use zircon_runtime::core::CoreError;
use zircon_runtime::plugin::{
    SceneRuntimeHook, SceneRuntimeHookContext, SceneRuntimeHookDescriptor,
    SceneRuntimeHookRegistration,
};
use zircon_runtime::scene::components::SystemStage;

#[derive(Clone, Debug, Default)]
pub struct PhysicsSceneRuntimeHook;

pub fn scene_hook_registration() -> SceneRuntimeHookRegistration {
    SceneRuntimeHookRegistration::new(
        SceneRuntimeHookDescriptor::new(
            "physics.scene.fixed_update",
            crate::PLUGIN_ID,
            SystemStage::FixedUpdate,
        ),
        PhysicsSceneRuntimeHook,
    )
}

impl SceneRuntimeHook for PhysicsSceneRuntimeHook {
    fn run(&self, context: SceneRuntimeHookContext<'_>) -> Result<(), CoreError> {
        let Ok(physics) = resolve_physics_manager(context.core) else {
            context
                .level
                .record_physics_step(PhysicsWorldStepPlan::default(), Vec::new());
            return Ok(());
        };

        let result = context.level.with_world_mut(|world| {
            physics.tick_scene_world(context.level.world_handle(), world, context.delta_seconds)
        });
        context
            .level
            .record_physics_step(result.step_plan, result.contacts);
        Ok(())
    }
}
