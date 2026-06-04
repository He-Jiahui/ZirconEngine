use super::super::super::super::*;

use super::ids::{EVENT_ID, PAYLOAD_SCHEMA};

pub(super) fn register_event(sound: &DefaultSoundManager) {
    sound
        .register_dynamic_event(SoundDynamicEventDescriptor {
            id: EVENT_ID.to_string(),
            display_name: "Weapon Fire".to_string(),
            payload_schema: PAYLOAD_SCHEMA.to_string(),
        })
        .unwrap();
}

pub(super) fn register_handlers(sound: &DefaultSoundManager) {
    for (plugin_id, handler_id, priority) in [
        ("timeline_sequence", "timeline-marker", 10),
        ("gameplay_audio", "weapon-foley", 20),
        ("analytics", "combat-counter", 20),
    ] {
        sound
            .register_dynamic_event_handler(SoundDynamicEventHandlerDescriptor {
                plugin_id: plugin_id.to_string(),
                handler_id: handler_id.to_string(),
                event_id: EVENT_ID.to_string(),
                display_name: handler_id.to_string(),
                priority,
            })
            .unwrap();
    }
}
