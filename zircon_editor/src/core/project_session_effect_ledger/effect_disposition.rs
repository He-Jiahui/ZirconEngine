//! 区分已准备、已提交、已回滚与需恢复的副作用状态；残留会话调查按状态而非动作名称推断安全补偿。

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProjectSessionEffectDisposition {
    Prepared,
    Committed,
    RolledBack,
    RecoveryRequired,
}
