// 使用清理夹具当前身份提交事件；调用方须先建立目录，测试才能单独观察后续卸载影响。
use super::super::super::super::*;

use super::fixture::CleanupFixture;

pub(crate) fn submit_cleanup_invocation(sound: &DefaultSoundManager, fixture: &CleanupFixture) {
    sound
        .submit_dynamic_event(SoundDynamicEventInvocation {
            event_id: fixture.event_id().to_string(),
            source_path: None,
            time_seconds: 0.0,
            payload_schema: fixture.payload_schema().to_string(),
            payload: Vec::new(),
        })
        .unwrap();
}
