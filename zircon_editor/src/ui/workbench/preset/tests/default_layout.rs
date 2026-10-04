use crate::ui::workbench::layout::{LayoutCommand, LayoutManager};
use crate::ui::workbench::view::ViewHost;

use super::*;

#[test]
fn default_layout_places_scene_game_documents_and_fyrox_panels() {
    let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
    let layout = stack.default_workbench_layout();

    let [MainHostPageLayout::WorkbenchPage { id, .. }] = layout.main_pages.as_slice() else {
        panic!("default layout should expose one main workbench page");
    };
    let document_workspace = layout
        .content_workspace_for_page(id)
        .expect("workbench page should resolve the workbench activity window");
    let DocumentNode::Tabs(documents) = document_workspace else {
        panic!("workbench document area should be a tab stack");
    };
    assert_eq!(
        documents.tabs,
        vec![
            ViewInstanceId::new("editor.scene#1"),
            ViewInstanceId::new("editor.game#1")
        ]
    );
    assert_eq!(
        documents.active_tab,
        Some(ViewInstanceId::new("editor.scene#1"))
    );

    assert_drawer_tabs(
        &layout,
        ActivityDrawerSlot::LeftTop,
        &["editor.hierarchy#1", "editor.assets#1"],
    );
    assert_drawer_tabs(
        &layout,
        ActivityDrawerSlot::RightTop,
        &["editor.inspector#1"],
    );
    assert_drawer_tabs(
        &layout,
        ActivityDrawerSlot::Bottom,
        &[
            "editor.console#1",
            "editor.runtime_diagnostics#1",
            "editor.build_export_desktop#1",
        ],
    );
    assert_drawer_tabs(
        &layout,
        ActivityDrawerSlot::LeftBottom,
        &["editor.module_plugins#1"],
    );
    assert_eq!(
        layout.active_activity_window_drawers()[&ActivityDrawerSlot::LeftTop].mode,
        ActivityDrawerMode::Pinned
    );
    assert_eq!(
        layout.active_activity_window_drawers()[&ActivityDrawerSlot::LeftBottom].mode,
        ActivityDrawerMode::Collapsed
    );
    assert_eq!(
        layout.active_activity_window_drawers()[&ActivityDrawerSlot::Bottom].mode,
        ActivityDrawerMode::Collapsed
    );
    assert_eq!(
        layout.activity_windows[&ActivityWindowId::workbench()].activity_drawers
            [&ActivityDrawerSlot::LeftBottom]
            .mode,
        ActivityDrawerMode::Collapsed
    );
}

#[test]
fn default_layout_registers_functional_windows_as_independent_units() {
    let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
    let layout = stack.default_workbench_layout();

    assert_eq!(
        layout.activity_windows.len(),
        stack.window_model.windows.len()
    );

    let material = layout
        .activity_windows
        .get(&ActivityWindowId::new("window:material_editor"))
        .expect("material editor activity window");
    assert_eq!(
        material.host_mode,
        ActivityWindowHostMode::NativeWindowHandle
    );
    let DocumentNode::Tabs(material_tabs) = &material.content_workspace else {
        panic!("material editor should use primary document tabs");
    };
    assert_eq!(
        material_tabs.tabs,
        vec![
            ViewInstanceId::new("editor.material.graph#material_editor"),
            ViewInstanceId::new("editor.material.preview#material_editor")
        ]
    );
    assert_drawer_tabs_in_window(
        material,
        ActivityDrawerSlot::RightTop,
        &["editor.inspector#material_editor"],
    );
    assert_drawer_tabs_in_window(
        material,
        ActivityDrawerSlot::LeftTop,
        &["editor.asset_browser#material_editor"],
    );
}

#[test]
fn preset_layout_supports_drawer_selection_detach_attach_and_focus() {
    let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
    let mut layout = stack.default_workbench_layout();
    let manager = LayoutManager;

    manager
        .apply(
            &mut layout,
            LayoutCommand::ActivateDrawerTab {
                slot: ActivityDrawerSlot::LeftTop,
                instance_id: ViewInstanceId::new("editor.assets#1"),
            },
        )
        .unwrap();
    assert_eq!(
        layout.active_activity_window_drawers()[&ActivityDrawerSlot::LeftTop].active_view,
        Some(ViewInstanceId::new("editor.assets#1"))
    );

    manager
        .apply(
            &mut layout,
            LayoutCommand::DetachViewToWindow {
                instance_id: ViewInstanceId::new("editor.scene#1"),
                new_window: MainPageId::new("floating:scene"),
            },
        )
        .unwrap();
    assert_eq!(layout.floating_windows.len(), 1);
    assert_eq!(
        layout.floating_windows[0].focused_view,
        Some(ViewInstanceId::new("editor.scene#1"))
    );

    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: ViewInstanceId::new("editor.scene#1"),
                target: ViewHost::Document(MainPageId::workbench(), vec![]),
                anchor: None,
            },
        )
        .unwrap();
    assert!(layout.floating_windows.is_empty());
    let MainHostPageLayout::WorkbenchPage { id, .. } = &layout.main_pages[0] else {
        panic!("main page should remain the workbench page");
    };
    let document_workspace = layout
        .content_workspace_for_page(id)
        .expect("main page should resolve its activity-window content workspace");
    assert!(document_workspace.contains(&ViewInstanceId::new("editor.scene#1")));
}

#[test]
fn preset_layout_focus_restores_collapsed_drawer_selection() {
    let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
    let mut layout = stack.default_workbench_layout();
    let manager = LayoutManager;

    assert_eq!(
        layout.active_activity_window_drawers()[&ActivityDrawerSlot::LeftBottom].mode,
        ActivityDrawerMode::Collapsed
    );

    manager
        .apply(
            &mut layout,
            LayoutCommand::FocusView {
                instance_id: ViewInstanceId::new("editor.module_plugins#1"),
            },
        )
        .unwrap();

    let activity_drawer = &layout.active_activity_window_drawers()[&ActivityDrawerSlot::LeftBottom];
    assert_eq!(activity_drawer.mode, ActivityDrawerMode::Pinned);
    assert_eq!(
        activity_drawer.active_view,
        Some(ViewInstanceId::new("editor.module_plugins#1"))
    );
}

#[test]
fn preset_layout_focus_restores_collapsed_output_selection() {
    let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
    let mut layout = stack.default_workbench_layout();
    let manager = LayoutManager;

    assert_eq!(
        layout.active_activity_window_drawers()[&ActivityDrawerSlot::Bottom].mode,
        ActivityDrawerMode::Collapsed
    );

    manager
        .apply(
            &mut layout,
            LayoutCommand::FocusView {
                instance_id: ViewInstanceId::new("editor.console#1"),
            },
        )
        .unwrap();

    let output = &layout.active_activity_window_drawers()[&ActivityDrawerSlot::Bottom];
    assert_eq!(output.mode, ActivityDrawerMode::Pinned);
    assert_eq!(
        output.active_view,
        Some(ViewInstanceId::new("editor.console#1"))
    );
}

fn assert_drawer_tabs(layout: &WorkbenchLayout, slot: ActivityDrawerSlot, expected: &[&str]) {
    assert_drawer_tabs_in_window(
        &layout.activity_windows[&ActivityWindowId::workbench()],
        slot,
        expected,
    );
}

fn assert_drawer_tabs_in_window(
    window: &ActivityWindowLayout,
    slot: ActivityDrawerSlot,
    expected: &[&str],
) {
    assert_drawer_tabs_in_map(&window.activity_drawers, slot, expected);
}

fn assert_drawer_tabs_in_map(
    drawers: &BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
    slot: ActivityDrawerSlot,
    expected: &[&str],
) {
    let expected = expected
        .iter()
        .copied()
        .map(ViewInstanceId::new)
        .collect::<Vec<_>>();
    assert_eq!(drawers[&slot].tab_stack.tabs, expected);
}
