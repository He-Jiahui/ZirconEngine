// 提交目录中不存在的事件，核对 UnknownDynamicEvent 这一注册前置条件。
use super::super::super::*;

use super::support::marker_invocation;

#[test]
fn dynamic_event_registry_rejects_unknown_invocation_event() {
    let sound = DefaultSoundManager::default();
    let mut invocation = marker_invocation();
    invocation.event_id = "sound.dynamic.missing".to_string();

    assert!(matches!(
        sound.submit_dynamic_event(invocation).unwrap_err(),
        SoundError::UnknownDynamicEvent { .. }
    ));
}
