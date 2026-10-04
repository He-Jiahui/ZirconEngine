use serde::{Deserialize, Serialize};

use crate::ui::workbench::layout::ActivityWindowId;
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

use super::DrawerDockPosition;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 抽屉实例的派生归属记录；跨窗口迁移须同步窗口列表与实例索引。
pub struct DrawerViewInstance {
    pub instance_id: ViewInstanceId,
    pub descriptor_id: ViewDescriptorId,
    pub title: String,
    pub owner_window: ActivityWindowId,
    pub dock_position: DrawerDockPosition,
}

impl DrawerViewInstance {
    pub fn new(
        instance_id: ViewInstanceId,
        descriptor_id: ViewDescriptorId,
        title: impl Into<String>,
        owner_window: ActivityWindowId,
        dock_position: DrawerDockPosition,
    ) -> Self {
        Self {
            instance_id,
            descriptor_id,
            title: title.into(),
            owner_window,
            dock_position,
        }
    }
}
