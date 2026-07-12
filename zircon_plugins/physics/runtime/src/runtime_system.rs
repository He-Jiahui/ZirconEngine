use zircon_runtime::core::framework::physics::{
    PhysicsContactEvent, PhysicsTriggerEvent, PhysicsWorldStepPlan, SimulatedPoseFeed,
    SkeletalPoseTargets,
};
use zircon_runtime::core::manager::resolve_physics_manager;
use zircon_runtime::core::CoreError;
use zircon_runtime::plugin::{PluginEventManifest, RuntimeExtensionRegistryError};
use zircon_runtime::scene::ecs::RuntimeSceneSystemContext;
use zircon_runtime::scene::SystemStage;

use crate::manager::apply_synchronized_bodies_to_scene;
use crate::record_physics_step_diagnostic;
use crate::skeletal::{
    drive_ragdoll_bodies_from_animation, write_simulated_pose_feed, RagdollRuntime,
};

#[derive(Clone, Debug, Default)]
pub struct PhysicsRuntimeSystem;

pub const PHYSICS_SYSTEM_SET: &str = "physics.main";
pub const PHYSICS_STEP_SYSTEM: &str = "physics.step";
pub const PHYSICS_SYNC_TO_SCENE_SYSTEM: &str = "physics.sync_to_scene";
pub const PHYSICS_CONTACT_EVENT_ID: &str = "physics.events.contact";
pub const PHYSICS_CONTACT_EVENT_SCHEMA: &str = "physics.contact_event.v1";
pub const PHYSICS_TRIGGER_EVENT_ID: &str = "physics.events.trigger";
pub const PHYSICS_TRIGGER_EVENT_SCHEMA: &str = "physics.trigger_event.v1";

pub fn register_runtime_systems(
    module: &mut zircon_plugin_sdk::RuntimePluginModuleRegistration<'_>,
) -> Result<(), RuntimeExtensionRegistryError> {
    module.event::<PhysicsContactEvent>(PluginEventManifest {
        id: PHYSICS_CONTACT_EVENT_ID.to_string(),
        display_name: "Physics Contact Event".to_string(),
        payload_schema: PHYSICS_CONTACT_EVENT_SCHEMA.to_string(),
    })?;
    module.event::<PhysicsTriggerEvent>(PluginEventManifest {
        id: PHYSICS_TRIGGER_EVENT_ID.to_string(),
        display_name: "Physics Trigger Event".to_string(),
        payload_schema: PHYSICS_TRIGGER_EVENT_SCHEMA.to_string(),
    })?;
    module.resource(SkeletalPoseTargets::default)?;
    module.resource(SimulatedPoseFeed::default)?;
    module.resource(RagdollRuntime::default)?;
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
    let started_at = Instant::now();
    let frame_index = context.core.real_time().frame_index();
    let Ok(physics) = resolve_physics_manager(context.core) else {
        context
            .level
            .record_physics_step(PhysicsWorldStepPlan::default(), Vec::new(), Vec::new());
        record_physics_step_diagnostic(context.core, frame_index, started_at.elapsed());
        return Ok(());
    };

    let result = context.level.with_world_mut(|world| {
        if let Some(mut ragdolls) = world.get_resource::<RagdollRuntime>().cloned() {
            drive_ragdoll_bodies_from_animation(world, &mut ragdolls, context.delta_seconds);
            if let Some(runtime) = world.get_resource_mut::<RagdollRuntime>() {
                *runtime = ragdolls;
            }
        }
        physics.tick_scene_world(context.level.world_handle(), world, context.delta_seconds)
    });
    context.level.with_world_mut(|world| {
        for contact in result.contacts.iter().cloned() {
            world.send_event(contact);
        }
        for trigger in result.triggers.iter().cloned() {
            world.send_event(trigger);
        }
    });
    context
        .level
        .record_physics_step(result.step_plan, result.contacts, result.triggers);
    record_physics_step_diagnostic(context.core, frame_index, started_at.elapsed());
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
    context.level.with_world_mut(|world| {
        apply_synchronized_bodies_to_scene(world, &sync);
        let Some(ragdolls) = world.get_resource::<RagdollRuntime>().cloned() else {
            return;
        };
        let interpolation_alpha = context
            .level
            .last_physics_step_plan()
            .map(|plan| {
                if plan.steps > 0 {
                    1.0
                } else {
                    plan.interpolation_alpha
                }
            })
            .unwrap_or(0.0);
        let mut next_feed = SimulatedPoseFeed::default();
        write_simulated_pose_feed(world, &sync, &ragdolls, interpolation_alpha, &mut next_feed);
        if let Some(feed) = world.get_resource_mut::<SimulatedPoseFeed>() {
            *feed = next_feed;
        }
    });
    Ok(())
}
use std::time::Instant;
