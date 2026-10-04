use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
/// 层级行的预览输入；选中状态在快照适配时提升到单独的选择集合。
pub struct PreviewSceneEntry {
    pub id: u64,
    pub name: String,
    pub depth: usize,
    pub selected: bool,
}
