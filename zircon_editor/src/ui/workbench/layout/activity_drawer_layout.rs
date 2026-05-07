use serde::{Deserialize, Serialize};

use crate::ui::workbench::view::ViewInstanceId;

use super::{ActivityDrawerMode, ActivityDrawerSlot, TabStackLayout};

const DEFAULT_SIDE_DRAWER_EXTENT: f32 = 260.0;
const DEFAULT_BOTTOM_DRAWER_EXTENT: f32 = 148.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActivityDrawerLayout {
    pub slot: ActivityDrawerSlot,
    pub tab_stack: TabStackLayout,
    pub active_view: Option<ViewInstanceId>,
    pub mode: ActivityDrawerMode,
    pub extent: f32,
    pub visible: bool,
}

impl ActivityDrawerLayout {
    pub fn new(slot: ActivityDrawerSlot) -> Self {
        Self {
            slot,
            tab_stack: TabStackLayout::default(),
            active_view: None,
            mode: ActivityDrawerMode::Pinned,
            extent: if slot.is_bottom() {
                DEFAULT_BOTTOM_DRAWER_EXTENT
            } else {
                DEFAULT_SIDE_DRAWER_EXTENT
            },
            visible: true,
        }
    }
}
