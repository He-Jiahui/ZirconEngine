use serde::{Deserialize, Serialize};

use crate::ViewInstanceId;

use super::{ActivityDrawerMode, ActivityDrawerSlot, TabStackLayout};

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
            extent: if matches!(
                slot,
                ActivityDrawerSlot::BottomLeft | ActivityDrawerSlot::BottomRight
            ) {
                200.0
            } else {
                260.0
            },
            visible: true,
        }
    }
}
