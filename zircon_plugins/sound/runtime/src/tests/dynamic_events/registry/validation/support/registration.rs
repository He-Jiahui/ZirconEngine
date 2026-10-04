// 汇集本组测试共享的有效描述符和入口调用，保证多个断言从一致的初始状态出发。
use super::super::super::super::*;

use super::ids::{EVENT_ID, PAYLOAD_SCHEMA};

pub(crate) fn register_marker_event(sound: &DefaultSoundManager) {
    sound
        .register_dynamic_event(SoundDynamicEventDescriptor {
            id: EVENT_ID.to_string(),
            display_name: "Marker".to_string(),
            payload_schema: PAYLOAD_SCHEMA.to_string(),
        })
        .unwrap();
}
