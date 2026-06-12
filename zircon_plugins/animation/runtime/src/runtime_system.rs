use zircon_runtime::core::CoreError;
use zircon_runtime::plugin::{
    PluginModuleId, RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
};
use zircon_runtime::scene::ecs::{RuntimeSceneSystemContext, SystemRef};
use zircon_runtime::scene::SystemStage;

#[path = "scene_hook/events.rs"]
mod events;
#[path = "scene_hook/graph.rs"]
mod graph;
#[path = "scene_hook/node_pose.rs"]
mod node_pose;
#[path = "scene_hook/pending.rs"]
mod pending;
#[path = "scene_hook/pose.rs"]
mod pose;
#[path = "scene_hook/scan.rs"]
mod scan;
#[path = "scene_hook/sequences.rs"]
mod sequences;
#[path = "scene_hook/state_machine.rs"]
mod state_machine;
#[path = "scene_hook/tick.rs"]
mod tick;

#[derive(Clone, Debug, Default)]
pub struct AnimationRuntimeSystem;

pub const ANIMATION_SYSTEM_SET: &str = "animation.evaluation";
pub const ANIMATION_EVALUATE_SYSTEM: &str = "animation.evaluate";

pub fn register_runtime_system(
    registry: &mut RuntimeExtensionRegistry,
    owner: PluginModuleId,
) -> Result<(), RuntimeExtensionRegistryError> {
    let animation_set = registry.intern_system_set(ANIMATION_SYSTEM_SET)?;
    registry
        .register_runtime_scene_system(
            owner,
            ANIMATION_EVALUATE_SYSTEM,
            SystemStage::PostUpdate,
            run_animation_runtime_system,
        )
        .in_set(animation_set)
        .after(SystemRef::System(
            "zircon.scene.world_transform".to_string(),
        ))
        .register()
}

fn run_animation_runtime_system(context: RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> {
    tick::tick_animation_world(context.core, context.level, context.delta_seconds);
    Ok(())
}
