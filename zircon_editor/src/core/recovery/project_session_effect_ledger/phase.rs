//! 项目会话台账的阶段身份；激活与关闭的副作用库存互斥，残留恢复按持久相位判断。

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProjectSessionEffectLedgerPhase {
    Activating,
    Ready,
    Closing,
    Closed,
    RecoveryRequired,
}
