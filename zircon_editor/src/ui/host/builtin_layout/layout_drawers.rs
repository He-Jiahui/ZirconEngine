use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, TabStackLayout,
};
use crate::ui::workbench::view::ViewInstanceId;

use super::super::editor_subsystems::{
    EditorSubsystemReport, EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
};

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
        tab_stack: TabStackLayout {
            tabs: vec![ViewInstanceId::new("editor.module_plugins#1")],
            active_tab: Some(ViewInstanceId::new("editor.module_plugins#1")),
        },
        active_view: Some(ViewInstanceId::new("editor.module_plugins#1")),
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

pub(super) fn bottom_right_drawer(subsystems: &EditorSubsystemReport) -> ActivityDrawerLayout {
    let diagnostics = ViewInstanceId::new("editor.runtime_diagnostics#1");
    let mut tabs = subsystems
        .is_enabled(EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS)
        .then_some(diagnostics.clone())
        .into_iter()
        .collect::<Vec<_>>();
    tabs.push(ViewInstanceId::new("editor.build_export_desktop#1"));
    let active = tabs.first().cloned();
    ActivityDrawerLayout {
        slot: ActivityDrawerSlot::BottomRight,
        tab_stack: TabStackLayout {
            tabs,
            active_tab: active.clone(),
        },
        active_view: active,
        mode: ActivityDrawerMode::Collapsed,
        extent: 224.0,
        visible: true,
    }
}
