use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 壳几何与尺寸覆盖的稳定身份；左右各包含两个工具slot，Document属于中心文档。
pub enum ShellRegionId {
    Left,
    Document,
    Right,
    Bottom,
}
