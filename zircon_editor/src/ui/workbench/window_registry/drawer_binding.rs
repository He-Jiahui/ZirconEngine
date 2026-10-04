use serde::{Deserialize, Serialize};

use crate::ui::workbench::layout::ActivityWindowId;
use crate::ui::workbench::view::ViewInstanceId;

use super::DrawerDockPosition;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 窗口注册表的重绑请求；派生索引更新不等于已修改权威布局。
pub struct DrawerBinding {
    pub window_id: ActivityWindowId,
    pub drawer_view: ViewInstanceId,
    pub dock_position: DrawerDockPosition,
}

impl DrawerBinding {
    pub fn new(
        window_id: ActivityWindowId,
        drawer_view: ViewInstanceId,
        dock_position: DrawerDockPosition,
    ) -> Self {
        Self {
            window_id,
            drawer_view,
            dock_position,
        }
    }
}
