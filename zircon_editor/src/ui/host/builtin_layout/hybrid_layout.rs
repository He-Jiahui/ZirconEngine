use std::collections::BTreeMap;

use crate::ui::workbench::layout::{
    ActivityDrawerSlot, ActivityWindowHostMode, ActivityWindowId, ActivityWindowLayout,
    DocumentNode, WorkbenchLayout,
};
use crate::ui::workbench::view::ViewDescriptorId;

use super::super::editor_subsystems::EditorSubsystemReport;
use super::layout_drawers::{
    bottom_drawer, left_bottom_drawer, left_top_drawer, right_bottom_drawer, right_top_drawer,
};
use super::workbench_page::builtin_workbench_page;

pub(crate) fn builtin_hybrid_layout() -> WorkbenchLayout {
    builtin_hybrid_layout_for_subsystems(&EditorSubsystemReport::default_enabled())
}

pub(crate) fn builtin_hybrid_layout_for_subsystems(
    subsystems: &EditorSubsystemReport,
) -> WorkbenchLayout {
    let drawers = BTreeMap::from([
        (ActivityDrawerSlot::LeftTop, left_top_drawer()),
        (ActivityDrawerSlot::LeftBottom, left_bottom_drawer()),
        (ActivityDrawerSlot::RightTop, right_top_drawer()),
        (ActivityDrawerSlot::RightBottom, right_bottom_drawer()),
        (ActivityDrawerSlot::Bottom, bottom_drawer(subsystems)),
    ]);
    WorkbenchLayout {
        active_main_page: crate::ui::workbench::layout::MainPageId::workbench(),
        main_pages: vec![builtin_workbench_page()],
        drawers: drawers.clone(),
        activity_windows: BTreeMap::from([(
            ActivityWindowId::workbench(),
            ActivityWindowLayout {
                window_id: ActivityWindowId::workbench(),
                descriptor_id: ViewDescriptorId::new("editor.workbench_window"),
                host_mode: ActivityWindowHostMode::EmbeddedMainFrame,
                activity_drawers: drawers,
                content_workspace: DocumentNode::default(),
                menu_overflow_mode: Default::default(),
                region_overrides: BTreeMap::new(),
                view_overrides: BTreeMap::new(),
            },
        )]),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    }
}
