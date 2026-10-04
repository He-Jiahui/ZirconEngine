use serde::{Deserialize, Serialize};

use crate::core::editor_message::{PlayStateKind, SceneModeId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 场景模式最新态与 Play 生命周期变化；后者逐条保留，供插件桥接识别进入和退出 Play。
pub enum ModeMessage {
    SceneModeChanged {
        mode: SceneModeId,
    },
    PlayStateChanged {
        from: PlayStateKind,
        to: PlayStateKind,
    },
}
