use serde::{Deserialize, Serialize};

use super::MainPageId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 布局命令所指向的文档树根；访问层分别从主页面和浮窗查找节点，后续路径相对该根解释。
pub enum WorkspaceTarget {
    MainPage(MainPageId),
    FloatingWindow(MainPageId),
}
