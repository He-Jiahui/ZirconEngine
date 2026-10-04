//! 只转换已确定为抽屉的槽位，保持注册表和公开能力描述一致。
use crate::ui::ActivityDrawerSlotPreference;

use crate::ui::workbench::layout::ActivityDrawerSlot;

/// 已通过上游抽屉槽筛选后调用，保持内部布局槽与公开能力一一对应。
pub(super) fn drawer_slot_preference(slot: ActivityDrawerSlot) -> ActivityDrawerSlotPreference {
    match slot {
        ActivityDrawerSlot::LeftTop => ActivityDrawerSlotPreference::LeftTop,
        ActivityDrawerSlot::LeftBottom => ActivityDrawerSlotPreference::LeftBottom,
        ActivityDrawerSlot::RightTop => ActivityDrawerSlotPreference::RightTop,
        ActivityDrawerSlot::RightBottom => ActivityDrawerSlotPreference::RightBottom,
        ActivityDrawerSlot::Bottom => ActivityDrawerSlotPreference::Bottom,
    }
}
