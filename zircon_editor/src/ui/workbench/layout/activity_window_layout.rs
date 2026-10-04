use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::workbench::autolayout::{PaneConstraintOverride, ShellRegionId};
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};
use crate::ui::workbench::window_registry::MenuOverflowMode;

use super::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowHostMode,
    ActivityWindowId, DocumentNode,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// 活动窗口自己的内容与抽屉布局；各窗口独立维护选择和尺寸覆盖。
pub struct ActivityWindowLayout {
    pub window_id: ActivityWindowId,
    pub descriptor_id: ViewDescriptorId,
    pub host_mode: ActivityWindowHostMode,
    pub activity_drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
    pub content_workspace: DocumentNode,
    pub menu_overflow_mode: MenuOverflowMode,
    pub region_overrides: BTreeMap<ShellRegionId, PaneConstraintOverride>,
    pub view_overrides: BTreeMap<ViewInstanceId, PaneConstraintOverride>,
}

impl ActivityWindowLayout {
    /// 激活一个抽屉前清同区域兄弟的选择；保留其成员以便后续恢复。
    pub(crate) fn collapse_drawer_region_siblings(
        &mut self,
        active_slot: ActivityDrawerSlot,
    ) -> bool {
        let mut changed = false;
        for (slot, drawer) in &mut self.activity_drawers {
            if *slot == active_slot || !slot.shares_region(active_slot) {
                continue;
            }
            let sibling_changed = drawer.mode != ActivityDrawerMode::Collapsed
                || drawer.tab_stack.active_tab.is_some()
                || drawer.active_view.is_some();
            if sibling_changed {
                drawer.mode = ActivityDrawerMode::Collapsed;
                drawer.tab_stack.active_tab = None;
                drawer.active_view = None;
                changed = true;
            }
        }
        changed
    }
}
