// 提供与注册目录匹配的空负载调用；拒绝用例只变更待验证字段，避免其他错误提前遮蔽结果。
use super::super::super::super::*;

use super::ids::{EVENT_ID, PAYLOAD_SCHEMA};

pub(crate) fn marker_invocation() -> SoundDynamicEventInvocation {
    SoundDynamicEventInvocation {
        event_id: EVENT_ID.to_string(),
        source_path: None,
        time_seconds: 0.0,
        payload_schema: PAYLOAD_SCHEMA.to_string(),
        payload: Vec::new(),
    }
}
