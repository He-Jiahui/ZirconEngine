use serde::{Deserialize, Serialize};

/// 启动属性对显示器的选择提示；索引属于当前后端枚举结果，不能当作持久显示器身份。
/// 创建首个窗口时 `Current` 没有已有窗口上下文，调用端须允许后端自行选择显示器。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowMonitorSelection {
    Current,
    #[default]
    Primary,
    Index(usize),
}

impl WindowMonitorSelection {
    pub const fn index(index: usize) -> Self {
        Self::Index(index)
    }
}
