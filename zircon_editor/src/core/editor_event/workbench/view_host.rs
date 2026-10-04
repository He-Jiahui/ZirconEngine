use serde::{Deserialize, Serialize};

use super::{ActivityDrawerSlot, MainPageId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 视图实例在工作台的宿主位置；文档与浮窗变体携带页面和树路径，布局附加与恢复需验证路径。
pub enum ViewHost {
    Drawer(ActivityDrawerSlot),
    Document(MainPageId, Vec<usize>),
    FloatingWindow(MainPageId, Vec<usize>),
    ExclusivePage(MainPageId),
}
