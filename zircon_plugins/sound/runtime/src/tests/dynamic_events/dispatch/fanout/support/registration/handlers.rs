use super::super::super::super::super::*;

use super::super::ids::EVENT_ID;

pub(crate) fn register_weapon_fire_handlers(sound: &DefaultSoundManager) {
    register_weapon_fire_handler(
        sound,
        "timeline_sequence",
        "timeline-marker",
        "Timeline Marker",
        10,
    );
    register_weapon_fire_handler(sound, "gameplay_audio", "weapon-foley", "Weapon Foley", 20);
    register_weapon_fire_handler(sound, "analytics", "combat-counter", "Combat Counter", 20);
}

fn register_weapon_fire_handler(
    sound: &DefaultSoundManager,
    plugin_id: &str,
    handler_id: &str,
    display_name: &str,
    priority: i32,
) {
    sound
        .register_dynamic_event_handler(SoundDynamicEventHandlerDescriptor {
            plugin_id: plugin_id.to_string(),
            handler_id: handler_id.to_string(),
            event_id: EVENT_ID.to_string(),
            display_name: display_name.to_string(),
            priority,
        })
        .unwrap();
}
