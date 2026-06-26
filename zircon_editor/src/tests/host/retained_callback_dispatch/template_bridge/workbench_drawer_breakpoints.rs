use super::super::support::*;
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;

#[test]
fn componentized_workbench_layout_collapses_right_drawer_shell_at_narrow_width() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let metrics = WorkbenchChromeMetrics::default();
    let mut bridge =
        match BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(640.0, 420.0)) {
            Ok(bridge) => bridge,
            Err(error) => panic!("workbench bridge should build: {error:?}"),
        };
    if let Err(error) =
        bridge.recompute_layout_with_workbench_model(UiSize::new(640.0, 420.0), &model, &metrics)
    {
        panic!("narrow workbench layout should recompute: {error:?}");
    }

    let narrow_frames = bridge.layout_frames();
    assert_eq!(narrow_frames.right_drawer_shell_frame, None);
    assert_eq!(narrow_frames.right_drawer_content_frame, None);

    if let Err(error) =
        bridge.recompute_layout_with_workbench_model(UiSize::new(900.0, 620.0), &model, &metrics)
    {
        panic!("regular workbench layout should recompute: {error:?}");
    }

    let regular_frames = bridge.layout_frames();
    let Some(right_shell_frame) = regular_frames.right_drawer_shell_frame else {
        panic!("right drawer shell should render at regular width");
    };
    let Some(right_content_frame) = regular_frames.right_drawer_content_frame else {
        panic!("right drawer content should render at regular width");
    };
    assert!(right_shell_frame.width > 0.0);
    assert!(right_content_frame.width > 0.0);
}
