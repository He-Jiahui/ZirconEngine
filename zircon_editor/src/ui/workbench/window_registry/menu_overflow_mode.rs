use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 窗口菜单的溢出策略请求；布局仍根据实际可用尺寸选择呈现。
pub enum MenuOverflowMode {
    #[default]
    Auto,
    Scroll,
    MultiColumn,
}
