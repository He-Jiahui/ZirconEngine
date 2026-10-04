use serde::{Deserialize, Serialize};

use super::MainPageId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 文档树路径的owner；同一子树路径在不同页面或浮动空间中各自解释。
pub enum WorkspaceTarget {
    MainPage(MainPageId),
    FloatingWindow(MainPageId),
}
