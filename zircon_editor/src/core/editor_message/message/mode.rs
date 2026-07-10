use serde::{Deserialize, Serialize};

use crate::core::editor_message::{PlayStateKind, SceneModeId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModeMessage {
    SceneModeChanged {
        mode: SceneModeId,
    },
    PlayStateChanged {
        from: PlayStateKind,
        to: PlayStateKind,
    },
}
