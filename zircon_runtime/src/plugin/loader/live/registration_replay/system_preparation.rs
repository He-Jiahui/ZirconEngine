//! Compile one replay system against the same manifest, bridge table and lifecycle.
use super::*;

pub(super) fn prepare_native_plugin_registration_system(
    plugin_id: &str,
    system: &NativePluginRegistrationSystem,
    manifest_capabilities: &[String],
    replay_context: &NativePluginRegistrationReplayBridgeContext,
    lifecycle: &RuntimePluginBridgeLifecycleState,
) -> NativePluginRegistrationReplayResult<PreparedNativePluginRegistrationSystem> {
    let stage = system.stage().map_err(|source| {
        NativePluginRegistrationReplayError::InvalidRegistrationSystem {
            plugin_id: plugin_id.to_string(),
            system_id: system.id.clone(),
            source,
        }
    })?;
    let access_plan = Arc::new(
        system
            .access_plan(manifest_capabilities)
            .map_err(
                |source| NativePluginRegistrationReplayError::InvalidRegistrationSystem {
                    plugin_id: plugin_id.to_string(),
                    system_id: system.id.clone(),
                    source,
                },
            )?,
    );
    let bridge_interface = system
        .bridge_interface()
        .map_err(
            |source| NativePluginRegistrationReplayError::InvalidRegistrationSystem {
                plugin_id: plugin_id.to_string(),
                system_id: system.id.clone(),
                source,
            },
        )?
        .to_string();
    let bridge_method = system
        .bridge_method()
        .map_err(
            |source| NativePluginRegistrationReplayError::InvalidRegistrationSystem {
                plugin_id: plugin_id.to_string(),
                system_id: system.id.clone(),
                source,
            },
        )?
        .to_string();
    let bridge_method_slot = replay_context
        .method_slot_result(plugin_id, &bridge_interface, &bridge_method)
        .map_err(
            |source| NativePluginRegistrationReplayError::BridgeMethodSlot {
                plugin_id: plugin_id.to_string(),
                system_id: system.id.clone(),
                bridge_interface: bridge_interface.clone(),
                bridge_method: bridge_method.clone(),
                source: source.to_string(),
            },
        )?;
    let bridge_interface_slot = lifecycle
        .bridge_table()
        .resolve_slot(&bridge_interface)
        .ok_or_else(
            || NativePluginRegistrationReplayError::UnknownBridgeInterface {
                plugin_id: plugin_id.to_string(),
                system_id: system.id.clone(),
                bridge_interface: bridge_interface.clone(),
            },
        )?;
    Ok(PreparedNativePluginRegistrationSystem {
        stage,
        access_plan,
        bridge_interface,
        bridge_method,
        bridge_interface_slot,
        bridge_method_slot,
    })
}
