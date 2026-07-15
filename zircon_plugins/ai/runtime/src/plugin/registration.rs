use std::sync::Arc;

use zircon_runtime::core::framework::ai::AiAgentTickReport;
use zircon_runtime::core::framework::script::ScriptBehaviorBridge;
use zircon_runtime::plugin::{
    PluginEventCatalogManifest, PluginEventManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError,
};

use crate::behavior_tree::{
    BehaviorNodeRegistry, BehaviorNodeRegistryService, RuntimeBehaviorIntegrationHost,
};
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
    let mut module = zircon_plugin_sdk::RuntimePluginRegistrationBuilder::new(registry)
        .module(AI_MODULE_NAME)?;
    let owner = module.owner();
    manager
        .bind_standard_behavior_nodes_to_owner(owner)
        .map_err(|error| {
            RuntimeExtensionRegistryError::InvalidPluginModule(format!(
                "{AI_MODULE_NAME}: behavior node catalog: {error}"
            ))
        })?;
    let node_registry: Arc<dyn BehaviorNodeRegistry> =
        Arc::new(BehaviorNodeRegistryService::new(manager.clone()));
    module.export_interface::<dyn BehaviorNodeRegistry>(node_registry)?;
    let revocation_manager = manager.clone();
    module.owner_revocation_listener(move |revoked_owner| {
        revocation_manager.revoke_behavior_node_owner(revoked_owner);
    });
    module.event::<AiAgentTickReport>(ai_tick_report_event())?;
    let mut frame = 0_u64;
    let script_bridge = module.import_interface::<dyn ScriptBehaviorBridge>()?;
    module
        .runtime_scene_system(
            AI_BEHAVIOR_TICK_SYSTEM,
            zircon_runtime::scene::SystemStage::Update,
            move |context| {
                let world_handle = context.level.world_handle();
                let active_entities = manager.active_agent_entities(world_handle);
                context.level.with_world_mut(|world| {
                    let camera_position = world
                        .world_transform(world.active_camera())
                        .map(|transform| transform.translation);
                    let lod_by_entity = active_entities
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
                        .collect::<std::collections::BTreeMap<_, _>>();
                    let mut integration_host =
                        RuntimeBehaviorIntegrationHost::new(world, Some(script_bridge.clone()));
                    let reports = manager
                        .tick_active_agents_with_lod_and_integration_host(
                            world_handle,
                            context.delta_seconds,
                            frame,
                            |entity| lod_by_entity.get(&entity).copied().unwrap_or_default(),
                            &mut integration_host,
                        )
                        .map_err(|error| {
                            zircon_runtime::core::CoreError::Initialization(
                                AI_BEHAVIOR_TICK_SYSTEM.to_string(),
                                error.to_string(),
                            )
                        })?;
                    drop(integration_host);
                    for report in reports {
                        world.send_event(report);
                    }
                    Ok(())
                })?;
                frame = frame.wrapping_add(1);
                Ok(())
            },
        )
        .register()
}
