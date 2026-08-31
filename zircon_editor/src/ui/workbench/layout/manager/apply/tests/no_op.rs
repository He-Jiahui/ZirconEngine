use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowId, LayoutCommand, LayoutCommandError,
    LayoutManager, MainPageId, SplitAxis, SplitPlacement, WorkbenchLayout, WorkspaceTarget,
};
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

#[test]
fn repeated_layout_commands_report_unchanged() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();

    assert!(
        !manager
            .apply(
                &mut layout,
                LayoutCommand::SetDrawerMode {
                    slot: ActivityDrawerSlot::LeftTop,
                    mode: ActivityDrawerMode::Pinned,
                },
            )
            .expect("drawer mode")
            .changed
    );
    assert!(
        !manager
            .apply(
                &mut layout,
                LayoutCommand::SetDrawerExtent {
                    slot: ActivityDrawerSlot::LeftTop,
                    extent: 260.0,
                },
            )
            .expect("drawer extent")
            .changed
    );
    assert!(
        !manager
            .apply(
                &mut layout,
                LayoutCommand::ActivateMainPage {
                    page_id: MainPageId::workbench(),
                },
            )
            .expect("main page")
            .changed
    );

    let instance_id = ViewInstanceId::new("editor.scene#performance");
    manager
        .apply(
            &mut layout,
            LayoutCommand::OpenView {
                instance_id: instance_id.clone(),
                target: ViewHost::Document(MainPageId::workbench(), Vec::new()),
            },
        )
        .expect("open view");
    assert!(
        !manager
            .apply(&mut layout, LayoutCommand::FocusView { instance_id })
            .expect("repeat focus")
            .changed
    );
}

#[test]
fn activating_a_missing_main_page_is_a_no_op() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let before = layout.clone();
    let missing_page = MainPageId::new("missing-page");

    let error = manager
        .apply(
            &mut layout,
            LayoutCommand::ActivateMainPage {
                page_id: missing_page.clone(),
            },
        )
        .expect_err("missing main page must be rejected before mutation");

    assert_eq!(
        error,
        LayoutCommandError::MissingMainPage {
            page_id: missing_page
        }
    );
    assert_eq!(layout, before);
}

#[test]
fn activating_a_page_with_a_missing_activity_window_is_a_no_op() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let window_id = ActivityWindowId::workbench();
    layout.activity_windows.remove(&window_id);
    let before = layout.clone();

    let error = manager
        .apply(
            &mut layout,
            LayoutCommand::ActivateMainPage {
                page_id: MainPageId::workbench(),
            },
        )
        .expect_err("page with no activity-window owner must be rejected");

    assert_eq!(
        error,
        LayoutCommandError::MissingActivityWindow {
            page_id: MainPageId::workbench(),
            window_id
        }
    );
    assert_eq!(layout, before);
}

#[test]
fn activating_a_duplicate_main_page_is_a_no_op() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    layout.main_pages.push(layout.main_pages[0].clone());
    let before = layout.clone();

    let error = manager
        .apply(
            &mut layout,
            LayoutCommand::ActivateMainPage {
                page_id: MainPageId::workbench(),
            },
        )
        .expect_err("duplicate main page identity must be rejected");

    assert_eq!(
        error,
        LayoutCommandError::DuplicateMainPage {
            page_id: MainPageId::workbench()
        }
    );
    assert_eq!(layout, before);
}

#[test]
fn resetting_an_already_default_layout_is_a_no_op() {
    let manager = LayoutManager::default();
    let mut layout = manager.default_layout();
    let before = layout.clone();

    let diff = manager
        .apply(&mut layout, LayoutCommand::ResetToDefault)
        .expect("default layout reset");

    assert!(!diff.changed);
    assert_eq!(layout, before);
}

#[test]
fn non_finite_geometry_commands_are_no_ops() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    manager
        .apply(
            &mut layout,
            LayoutCommand::CreateSplit {
                workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
                path: Vec::new(),
                axis: SplitAxis::Horizontal,
                placement: SplitPlacement::After,
                new_instance: ViewInstanceId::new("editor.preview#geometry"),
            },
        )
        .expect("split fixture");

    let before_split = layout.clone();
    let split_error = manager
        .apply(
            &mut layout,
            LayoutCommand::ResizeSplit {
                workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
                path: Vec::new(),
                ratio: f32::NAN,
            },
        )
        .expect_err("non-finite split ratio must be rejected");
    assert_eq!(
        split_error,
        LayoutCommandError::NonFiniteSplitRatio {
            workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
            path: Vec::new()
        }
    );
    assert_eq!(layout, before_split);

    let before_drawer = layout.clone();
    let drawer_error = manager
        .apply(
            &mut layout,
            LayoutCommand::SetDrawerExtent {
                slot: ActivityDrawerSlot::LeftTop,
                extent: f32::INFINITY,
            },
        )
        .expect_err("non-finite drawer extent must be rejected");
    assert_eq!(
        drawer_error,
        LayoutCommandError::NonFiniteDrawerExtent {
            slot: ActivityDrawerSlot::LeftTop
        }
    );
    assert_eq!(layout, before_drawer);
}
