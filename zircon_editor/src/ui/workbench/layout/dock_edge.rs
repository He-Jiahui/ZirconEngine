use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 几何命中的停靠边缘；转换后才成为布局分割方向及插入顺序。
pub enum DockEdge {
    Left,
    Right,
    Top,
    Bottom,
}
