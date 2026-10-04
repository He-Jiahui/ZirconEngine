use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 新空间相对原节点的子树顺序；具体方向由分割轴解释。
pub enum SplitPlacement {
    Before,
    After,
}
