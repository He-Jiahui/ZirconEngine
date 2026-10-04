use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 文档工作区分割方向；CreateSplit 和布局节点共用，使拖放目标与恢复后的结构一致。
pub enum SplitAxis {
    Horizontal,
    Vertical,
}
