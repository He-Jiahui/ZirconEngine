use super::super::super::super::*;

use super::ids::{EVENT_ID, PAYLOAD_SCHEMA};

pub(crate) fn submit_ambient_event(sound: &DefaultSoundManager) {
    sound
        .submit_dynamic_event(SoundDynamicEventInvocation {
            event_id: EVENT_ID.to_string(),
            source_path: None,
            time_seconds: 0.0,
            payload_schema: PAYLOAD_SCHEMA.to_string(),
            payload: Vec::new(),
        })
        .unwrap();
}
