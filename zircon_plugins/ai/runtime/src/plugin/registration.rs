use std::sync::Arc;

use zircon_runtime::core::framework::ai::AiAgentTickReport;
use zircon_runtime::plugin::{
    PluginEventCatalogManifest, PluginEventManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError,
};

use crate::behavior_tree::{BehaviorNodeRegistry, BehaviorNodeRegistryService};
use crate::{AiBehaviorTickLod, DefaultAiManager, AI_MODULE_NAME};

pub const AI_BEHAVIOR_TICK_SYSTEM: &str = "ai.behavior_tick";
pub const AI_EVENT_NAMESPACE: &str = "ai.events";

pub(super) fn ai_event_catalog() -> PluginEventCatalogManifest {
    PluginEventCatalogManifest {
        namespace: AI_EVENT_NAMESPACE.to_string(),
        version: 1,
        events: vec![ai_tick_report_event()],
    }
}

fn ai_tick_report_event() -> PluginEventManifest {
    PluginEventManifest {
        id: "ai.events.agent_tick_completed".to_string(),
        display_name: "AI Agent Tick Completed".to_string(),
        payload_schema: "ai.events.agent_tick_report.v1".to_string(),
    }
}

pub(super) fn register_runtime_extensions(
    registry: &mut RuntimeExtensionRegistry,
    manager: Arc<DefaultAiManager>,
) -> Result<(), RuntimeExtensionRegistryError> {
    let owner = registry.intern_plugin_module(AI_MODULE_NAME)?;
    manager
        .bind_standard_behavior_nodes_to_owner(owner)
        .map_err(|error| {
            RuntimeExtensionRegistryError::InvalidPluginModule(format!(
                "{AI_MODULE_NAME}: behavior node catalog: {error}"
            ))
        })?;
    let node_registry: Arc<dyn BehaviorNodeRegistry> =
        Arc::new(BehaviorNodeRegistryService::new(manager.clone()));
    registry.export_interface::<dyn BehaviorNodeRegistry>(owner, node_registry)?;
    let revocation_manager = manager.clone();
    registry.register_owner_revocation_listener(owner, move |revoked_owner| {
        revocation_manager.revoke_behavior_node_owner(revoked_owner);
    });
    registry.register_event::<AiAgentTickReport>(owner, ai_tick_report_event())?;
    let mut frame = 0_u64;
    registry
        .register_runtime_scene_system(
            owner,
            AI_BEHAVIOR_TICK_SYSTEM,
            zircon_runtime::scene::SystemStage::Update,
            move |context| {
                let world_handle = context.level.world_handle();
                let active_entities = manager.active_agent_entities(world_handle);
                let lod_by_entity = context.level.with_world(|world| {
                    let camera_position = world
                        .world_transform(world.active_camera())
                        .map(|transform| transform.translation);
                    active_entities
                        .iter()
                        .copied()
                        .map(|entity| {
                            let lod = camera_position
                                .zip(
                                    world
                                        .world_transform(entity)
                                        .map(|transform| transform.translation),
                                )
                                .map(|(camera, agent)| {
                                    AiBehaviorTickLod::from_distance((agent - camera).length())
                                })
                                .unwrap_or(AiBehaviorTickLod::Full);
                            (entity, lod)
                        })
                        .collect::<std::collections::BTreeMap<_, _>>()
                });
                let reports = manager
                    .tick_active_agents_with_lod(
                        world_handle,
                        context.delta_seconds,
                        frame,
                        |entity| lod_by_entity.get(&entity).copied().unwrap_or_default(),
                    )
                    .map_err(|error| {
                        zircon_runtime::core::CoreError::Initialization(
                            AI_BEHAVIOR_TICK_SYSTEM.to_string(),
                            error.to_string(),
                        )
                    })?;
                context.level.with_world_mut(|world| {
                    for report in reports {
                        world.send_event(report);
                    }
                });
                frame = frame.wrapping_add(1);
                Ok(())
            },
        )
        .register()
}
