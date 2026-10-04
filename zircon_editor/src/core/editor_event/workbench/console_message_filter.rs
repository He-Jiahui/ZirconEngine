use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 菜单和活动日志共用的严重度选择；投影层把 Info/All 解释为最低 Info，as_str 供稳定 UI 状态标识。
pub enum ConsoleMessageFilter {
    #[default]
    All,
    Info,
    Warning,
    Error,
}

impl ConsoleMessageFilter {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}
