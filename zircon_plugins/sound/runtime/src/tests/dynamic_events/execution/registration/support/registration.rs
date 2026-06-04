use super::super::super::super::*;

use super::ids::{EVENT_ID, HANDLER_ID, PAYLOAD_SCHEMA, PLUGIN_ID};

pub(crate) fn register_dynamic_event_handler(sound: &DefaultSoundManager) {
    sound
        .register_dynamic_event(SoundDynamicEventDescriptor {
            id: EVENT_ID.to_string(),
            display_name: "Registered".to_string(),
            payload_schema: PAYLOAD_SCHEMA.to_string(),
        })
        .unwrap();
    sound
        .register_dynamic_event_handler(SoundDynamicEventHandlerDescriptor {
            plugin_id: PLUGIN_ID.to_string(),
            handler_id: HANDLER_ID.to_string(),
            event_id: EVENT_ID.to_string(),
            display_name: "Registered Handler".to_string(),
            priority: 0,
        })
        .unwrap();
}
