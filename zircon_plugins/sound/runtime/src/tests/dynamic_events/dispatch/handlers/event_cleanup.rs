use super::support::{register_ambient_event, register_ambient_handler, submit_ambient_event};

use super::super::super::*;

#[test]
fn dynamic_event_unregister_removes_handlers_and_pending_dispatches() {
    let sound = DefaultSoundManager::default();
    register_ambient_event(&sound);
    register_ambient_handler(&sound);
    submit_ambient_event(&sound);

    sound
        .unregister_dynamic_event("sound.dynamic.ambient.stinger")
        .unwrap();

    assert!(sound.dynamic_event_handlers().unwrap().is_empty());
    assert!(sound.dispatch_dynamic_events().unwrap().is_empty());
}
