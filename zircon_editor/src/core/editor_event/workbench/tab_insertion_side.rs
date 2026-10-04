use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 与标签实例锚点配套的相对插入方向，供抽屉和文档标签复用。
pub enum TabInsertionSide {
    Before,
    After,
}
