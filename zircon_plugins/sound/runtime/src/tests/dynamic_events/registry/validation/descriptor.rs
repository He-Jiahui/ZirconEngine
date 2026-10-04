// 经事件注册入口拒绝空事件身份，核对描述符的身份前置条件。
use super::super::super::*;

#[test]
fn dynamic_event_registry_rejects_invalid_descriptor() {
    let sound = DefaultSoundManager::default();

    assert!(sound
        .register_dynamic_event(SoundDynamicEventDescriptor {
            id: String::new(),
            display_name: "Invalid".to_string(),
            payload_schema: "sound.dynamic.invalid.v1".to_string(),
        })
        .unwrap_err()
        .to_string()
        .contains("descriptor"));
}
