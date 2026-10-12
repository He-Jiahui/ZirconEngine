// 报告用例提交一份有效事件，之后按处理器展开为多个结果；重复执行用于核对队列已排空。
use super::super::super::super::*;

use super::ids::{EVENT_ID, PAYLOAD_SCHEMA};

pub(super) fn submit_event(sound: &DefaultSoundManager) {
    sound
        .submit_dynamic_event(SoundDynamicEventInvocation {
            event_id: EVENT_ID.to_string(),
            source_path: Some("Timeline/Combat/Weapon".to_string()),
            time_seconds: 4.0,
            payload_schema: PAYLOAD_SCHEMA.to_string(),
            payload: vec![7, 9],
        })
        .unwrap();
}
