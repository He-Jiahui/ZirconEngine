use serde::{Deserialize, Serialize};

/// 触发配对的进入、持续与退出阶段，供场景事件消费者维护交互状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicsTriggerEventKind {
    Enter,
    Stay,
    Exit,
}
