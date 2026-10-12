// 经处理器注销入口核对首次移除成功、重复移除返回 UnknownDynamicEventHandler。
use super::super::super::*;

use super::support::{register_ambient_event, register_ambient_handler};

#[test]
fn dynamic_event_handlers_unregister_and_report_unknown_handler() {
    let sound = DefaultSoundManager::default();
    register_ambient_event(&sound);
    register_ambient_handler(&sound);

    sound
        .unregister_dynamic_event_handler("ambience", "stinger")
        .unwrap();
    assert!(matches!(
        sound
            .unregister_dynamic_event_handler("ambience", "stinger")
            .unwrap_err(),
        SoundError::UnknownDynamicEventHandler { .. }
    ));
}
