// 经公共声管理器核对已提交待分发事件的排空结果，重复排空应返回空集合。
use super::super::super::*;

use super::support::{impact_invocation, register_impact_event};

#[test]
fn dynamic_event_registry_drains_pending_invocations() {
    let sound = DefaultSoundManager::default();
    register_impact_event(&sound);

    let invocation = impact_invocation();
    sound.submit_dynamic_event(invocation.clone()).unwrap();

    assert_eq!(sound.drain_dynamic_events().unwrap(), vec![invocation]);
    assert!(sound.drain_dynamic_events().unwrap().is_empty());
}
