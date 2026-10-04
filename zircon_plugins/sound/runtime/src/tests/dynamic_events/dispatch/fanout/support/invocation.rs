// 固定同一带负载事件，供多个处理器的内容一致性、排序与排空用例复用。
use super::super::super::super::*;

use super::ids::{EVENT_ID, PAYLOAD_SCHEMA};

pub(super) fn weapon_fire_invocation() -> SoundDynamicEventInvocation {
    SoundDynamicEventInvocation {
        event_id: EVENT_ID.to_string(),
        source_path: Some("Timeline/Combat/Weapon".to_string()),
        time_seconds: 4.0,
        payload_schema: PAYLOAD_SCHEMA.to_string(),
        payload: vec![7, 9],
    }
}
