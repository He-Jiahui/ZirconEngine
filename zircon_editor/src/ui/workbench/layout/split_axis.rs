use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 两个子工作区的排列方向；比例和尺寸约束属于空间节点及布局解算。
pub enum SplitAxis {
    Horizontal,
    Vertical,
}
