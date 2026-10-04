// 扇出用例先注册共同事件及负载模式，再注册处理器；此助手要求新的测试管理器状态。
use super::super::super::super::super::*;

use super::super::ids::{EVENT_ID, PAYLOAD_SCHEMA};

pub(crate) fn register_weapon_fire_event(sound: &DefaultSoundManager) {
    sound
        .register_dynamic_event(SoundDynamicEventDescriptor {
            id: EVENT_ID.to_string(),
            display_name: "Weapon Fire".to_string(),
            payload_schema: PAYLOAD_SCHEMA.to_string(),
        })
        .unwrap();
}
