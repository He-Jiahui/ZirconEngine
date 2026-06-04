use super::super::super::super::*;

use std::sync::{Arc, Mutex};

pub(super) fn register_executors(sound: &DefaultSoundManager, calls: &Arc<Mutex<Vec<String>>>) {
    let analytics_calls = calls.clone();
    sound
        .register_dynamic_event_executor("analytics", "combat-counter", move |delivery| {
            analytics_calls
                .lock()
                .unwrap()
                .push(delivery.handler.plugin_id.clone());
            Ok(())
        })
        .unwrap();
    let gameplay_calls = calls.clone();
    sound
        .register_dynamic_event_executor("gameplay_audio", "weapon-foley", move |delivery| {
            gameplay_calls
                .lock()
                .unwrap()
                .push(delivery.handler.plugin_id.clone());
            Err("foley unavailable".to_string())
        })
        .unwrap();
}
