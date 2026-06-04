use super::super::super::*;

use super::detail::{EVENT_ID, PAYLOAD_SCHEMA};

pub(crate) fn register_abi_event_and_handler(
    sound: &DefaultSoundManager,
    plugin_id: &str,
    handler_id: &str,
) {
    sound
        .register_dynamic_event(SoundDynamicEventDescriptor {
            id: EVENT_ID.to_string(),
            display_name: "ABI Event".to_string(),
            payload_schema: PAYLOAD_SCHEMA.to_string(),
        })
        .unwrap();
    sound
        .register_dynamic_event_handler(SoundDynamicEventHandlerDescriptor {
            plugin_id: plugin_id.to_string(),
            handler_id: handler_id.to_string(),
            event_id: EVENT_ID.to_string(),
            display_name: "ABI Handler".to_string(),
            priority: 0,
        })
        .unwrap();
}
