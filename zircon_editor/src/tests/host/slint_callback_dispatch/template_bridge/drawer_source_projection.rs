use super::super::support::*;
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;

#[test]
fn builtin_workbench_drawer_source_template_bridge_exports_visible_drawer_frames_from_workbench_model(
) {
    let _guard = env_lock().lock().unwrap();

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let mut bridge =
        BuiltinWorkbenchDrawerSourceTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(
            UiSize::new(1280.0, 720.0),
            &model,
            &WorkbenchChromeMetrics::default(),
        )
        .unwrap();

    let frames = bridge.source_frames();
    let center_height = 720.0 - 40.0 - 24.0 - 1.0 - 164.0;
    assert_eq!(
        frames.left_drawer_shell_frame,
        Some(UiFrame::new(0.0, 40.0, 312.0, center_height))
    );
    assert_eq!(
        frames.left_drawer_header_frame,
        Some(UiFrame::new(35.0, 40.0, 277.0, 25.0))
    );
    assert_eq!(
        frames.left_drawer_content_frame,
        Some(UiFrame::new(35.0, 66.0, 277.0, center_height - 26.0))
    );
    assert_eq!(
        frames.right_drawer_shell_frame,
        Some(UiFrame::new(972.0, 40.0, 308.0, center_height))
    );
    assert_eq!(
        frames.right_drawer_header_frame,
        Some(UiFrame::new(972.0, 40.0, 273.0, 25.0))
    );
    assert_eq!(
        frames.right_drawer_content_frame,
        Some(UiFrame::new(972.0, 66.0, 273.0, center_height - 26.0))
    );
    assert_eq!(
        frames.bottom_drawer_shell_frame,
        Some(UiFrame::new(0.0, 532.0, 1280.0, 164.0))
    );
    assert_eq!(
        frames.bottom_drawer_header_frame,
        Some(UiFrame::new(0.0, 532.0, 1280.0, 25.0))
    );
    assert_eq!(
        frames.bottom_drawer_content_frame,
        Some(UiFrame::new(0.0, 558.0, 1280.0, 138.0))
    );
}
