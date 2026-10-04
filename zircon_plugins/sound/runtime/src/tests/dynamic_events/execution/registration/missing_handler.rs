// 执行器注册必须引用已有处理器；本用例核对缺失处理器返回 UnknownDynamicEventHandler。
use super::super::super::*;

#[test]
fn dynamic_event_executor_registration_rejects_missing_handler() {
    let sound = DefaultSoundManager::default();

    assert!(matches!(
        sound
            .register_dynamic_event_executor("missing", "handler", |_| Ok(()))
            .unwrap_err(),
        SoundError::UnknownDynamicEventHandler { .. }
    ));
}
