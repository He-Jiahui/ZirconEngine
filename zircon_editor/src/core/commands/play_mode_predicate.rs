//! 在命令声明中表达运行模式门槛，供谓词求值与快捷键冲突域推导共用；声明匹配当前状态不等于触发运行模式切换。

use serde::{Deserialize, Serialize};

use crate::core::editor_message::PlayStateKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PlayModePredicate {
    Edit,
    Building,
    Playing,
    CleanupFailed,
}

impl PlayModePredicate {
    pub(crate) fn matches(self, state: PlayStateKind) -> bool {
        matches!(
            (self, state),
            (Self::Edit, PlayStateKind::Edit)
                | (Self::Building, PlayStateKind::Building)
                | (Self::Playing, PlayStateKind::Playing)
                | (Self::CleanupFailed, PlayStateKind::CleanupFailed)
        )
    }
}
