use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

use crate::ui::workbench::view::ViewInstanceId;

use super::{ActivityDrawerMode, ActivityDrawerSlot, TabStackLayout};

const DEFAULT_SIDE_DRAWER_EXTENT: f32 = 260.0;
const DEFAULT_BOTTOM_DRAWER_EXTENT: f32 = 148.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// 单个活动窗口的抽屉布局；尺寸是偏好，选中项须与tab栈及显示模式一致。
pub struct ActivityDrawerLayout {
    pub slot: ActivityDrawerSlot,
    pub tab_stack: TabStackLayout,
    pub active_view: Option<ViewInstanceId>,
    pub mode: ActivityDrawerMode,
    pub extent: f32,
    pub visible: bool,
}

impl ActivityDrawerLayout {
    /// 建立空但可见的抽屉基线；底部与侧边使用不同的初始尺寸偏好。
    pub fn new(slot: ActivityDrawerSlot) -> Self {
        Self {
            slot,
            tab_stack: TabStackLayout::default(),
            active_view: None,
            mode: ActivityDrawerMode::Pinned,
            extent: match slot {
                ActivityDrawerSlot::Bottom => DEFAULT_BOTTOM_DRAWER_EXTENT,
                ActivityDrawerSlot::RightTop | ActivityDrawerSlot::RightBottom => {
                    let tokens = EditorDesignTokens::workbench_dark();
                    tokens.density.right_drawer_width + tokens.chrome.activity_rail_width
                }
                ActivityDrawerSlot::LeftTop | ActivityDrawerSlot::LeftBottom => {
                    DEFAULT_SIDE_DRAWER_EXTENT
                }
            },
            visible: true,
        }
    }
}

#[cfg(test)]
#[path = "tests/activity_drawer_layout.rs"]
mod tests;
