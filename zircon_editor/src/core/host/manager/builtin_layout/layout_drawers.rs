use crate::layout::{ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, TabStackLayout};
use crate::view::ViewInstanceId;

pub(super) fn left_top_drawer() -> ActivityDrawerLayout {
    ActivityDrawerLayout {
        slot: ActivityDrawerSlot::LeftTop,
        tab_stack: TabStackLayout {
            tabs: vec![
                ViewInstanceId::new("editor.project#1"),
                ViewInstanceId::new("editor.assets#1"),
                ViewInstanceId::new("editor.hierarchy#1"),
            ],
            active_tab: Some(ViewInstanceId::new("editor.project#1")),
        },
        active_view: Some(ViewInstanceId::new("editor.project#1")),
        mode: ActivityDrawerMode::Pinned,
        extent: 312.0,
        visible: true,
    }
}

pub(super) fn left_bottom_drawer() -> ActivityDrawerLayout {
    ActivityDrawerLayout {
        slot: ActivityDrawerSlot::LeftBottom,
        tab_stack: TabStackLayout::default(),
        active_view: None,
        mode: ActivityDrawerMode::Collapsed,
        extent: 288.0,
        visible: true,
    }
}

pub(super) fn right_top_drawer() -> ActivityDrawerLayout {
    ActivityDrawerLayout {
        slot: ActivityDrawerSlot::RightTop,
        tab_stack: TabStackLayout {
            tabs: vec![ViewInstanceId::new("editor.inspector#1")],
            active_tab: Some(ViewInstanceId::new("editor.inspector#1")),
        },
        active_view: Some(ViewInstanceId::new("editor.inspector#1")),
        mode: ActivityDrawerMode::Pinned,
        extent: 308.0,
        visible: true,
    }
}

pub(super) fn right_bottom_drawer() -> ActivityDrawerLayout {
    ActivityDrawerLayout {
        slot: ActivityDrawerSlot::RightBottom,
        tab_stack: TabStackLayout::default(),
        active_view: None,
        mode: ActivityDrawerMode::Collapsed,
        extent: 288.0,
        visible: true,
    }
}

pub(super) fn bottom_left_drawer() -> ActivityDrawerLayout {
    ActivityDrawerLayout {
        slot: ActivityDrawerSlot::BottomLeft,
        tab_stack: TabStackLayout {
            tabs: vec![ViewInstanceId::new("editor.console#1")],
            active_tab: Some(ViewInstanceId::new("editor.console#1")),
        },
        active_view: Some(ViewInstanceId::new("editor.console#1")),
        mode: ActivityDrawerMode::Pinned,
        extent: 164.0,
        visible: true,
    }
}

pub(super) fn bottom_right_drawer() -> ActivityDrawerLayout {
    ActivityDrawerLayout {
        slot: ActivityDrawerSlot::BottomRight,
        tab_stack: TabStackLayout::default(),
        active_view: None,
        mode: ActivityDrawerMode::Collapsed,
        extent: 224.0,
        visible: true,
    }
}
