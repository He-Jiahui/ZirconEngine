use super::super::super::*;

use super::support::{register_ambient_event, register_ambient_handler};

#[test]
fn dynamic_event_handlers_require_registered_event_ownership() {
    let sound = DefaultSoundManager::default();
    assert!(matches!(
        sound
            .register_dynamic_event_handler(SoundDynamicEventHandlerDescriptor {
                plugin_id: "timeline_sequence".to_string(),
                handler_id: "missing-event".to_string(),
                event_id: "sound.dynamic.missing".to_string(),
                display_name: "Missing Event".to_string(),
                priority: 0,
            })
            .unwrap_err(),
        SoundError::UnknownDynamicEvent { .. }
    ));

    register_ambient_event(&sound);
    register_ambient_handler(&sound);

    assert_eq!(sound.dynamic_event_handlers().unwrap().len(), 1);
}
