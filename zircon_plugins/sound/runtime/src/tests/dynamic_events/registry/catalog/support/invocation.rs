use super::super::super::super::*;

use super::ids::{EVENT_ID, PAYLOAD_SCHEMA};

pub(crate) fn impact_invocation() -> SoundDynamicEventInvocation {
    SoundDynamicEventInvocation {
        event_id: EVENT_ID.to_string(),
        source_path: Some("Timeline/Combat/Impact".to_string()),
        time_seconds: 1.25,
        payload_schema: PAYLOAD_SCHEMA.to_string(),
        payload: vec![1, 2, 3, 4],
    }
}
