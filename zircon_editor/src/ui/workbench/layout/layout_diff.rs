use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 宿主刷新门槛：是否发生布局变化；不包含具体节点差异或事务状态。
pub struct LayoutDiff {
    pub changed: bool,
}
