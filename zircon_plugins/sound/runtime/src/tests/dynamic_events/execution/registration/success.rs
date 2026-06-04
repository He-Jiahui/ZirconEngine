use super::super::super::*;

use super::support::{register_dynamic_event_handler, HANDLER_ID, PLUGIN_ID};

#[test]
fn dynamic_event_executor_registration_accepts_registered_handler() {
    let sound = DefaultSoundManager::default();
    register_dynamic_event_handler(&sound);

    sound
        .register_dynamic_event_executor(PLUGIN_ID, HANDLER_ID, |_| Ok(()))
        .unwrap();
}
