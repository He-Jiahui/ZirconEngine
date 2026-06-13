use zircon_runtime::core::manager::resolve_net_manager;
use zircon_runtime::core::CoreError;
use zircon_runtime::plugin::{
    PluginEventManifest, PluginModuleId, RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
};
use zircon_runtime::scene::ecs::RuntimeSceneSystemContext;
use zircon_runtime::scene::SystemStage;

pub const NET_SYSTEM_SET: &str = "net.transport";
pub const NET_POLL_INGRESS_SYSTEM: &str = "net.poll_ingress";
pub const NET_FLUSH_EGRESS_SYSTEM: &str = "net.flush_egress";
pub const NET_EVENT_ID: &str = "net.events.runtime_event";
pub const NET_EVENT_SCHEMA: &str = "net.runtime_event.v1";
const NET_POLL_INGRESS_EVENT_BUDGET: usize = 256;

pub fn register_runtime_systems(
    registry: &mut RuntimeExtensionRegistry,
    owner: PluginModuleId,
) -> Result<(), RuntimeExtensionRegistryError> {
    let transport_set = registry.intern_system_set(NET_SYSTEM_SET)?;
    registry.register_event::<zircon_runtime::core::framework::net::NetEvent>(
        owner,
        PluginEventManifest {
            id: NET_EVENT_ID.to_string(),
            display_name: "Network Runtime Event".to_string(),
            payload_schema: NET_EVENT_SCHEMA.to_string(),
        },
    )?;
    registry
        .register_runtime_scene_system(
            owner,
            NET_POLL_INGRESS_SYSTEM,
            SystemStage::First,
            run_net_poll_ingress,
        )
        .in_set(transport_set.clone())
        .register()?;
    registry
        .register_runtime_scene_system(
            owner,
            NET_FLUSH_EGRESS_SYSTEM,
            SystemStage::Last,
            run_net_flush_egress,
        )
        .in_set(transport_set)
        .register()
}

fn run_net_poll_ingress(context: RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> {
    let Ok(net) = resolve_net_manager(context.core) else {
        return Ok(());
    };
    let events = net.drain_events(NET_POLL_INGRESS_EVENT_BUDGET);
    if events.is_empty() {
        return Ok(());
    }

    context.level.with_world_mut(|world| {
        for event in events {
            world.send_event(event);
        }
    });
    Ok(())
}

fn run_net_flush_egress(_context: RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> {
    Ok(())
}
