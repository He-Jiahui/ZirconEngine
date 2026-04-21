use crate::ui::host::NativeWindowHostState;
use crate::ui::slint_host::floating_window_projection::build_floating_window_projection_bundle;
use crate::ui::workbench::autolayout::{
    compute_workbench_shell_geometry, ShellSizePx, WorkbenchChromeMetrics,
};
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::support::floating_preview_fixture;

#[test]
fn native_floating_window_targets_fall_back_to_shared_geometry_when_host_bounds_are_empty() {
    let window_id = MainPageId::new("window:native-preview");
    let fixture = floating_preview_fixture(&window_id);
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(1440.0, 900.0),
        &WorkbenchChromeMetrics::default(),
        None,
    );

    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        &[NativeWindowHostState {
            window_id: window_id.clone(),
            handle: None,
            bounds: [0.0, 0.0, 0.0, 0.0],
        }],
    );
    let targets = crate::ui::slint_host::collect_native_floating_window_targets(
        &model,
        &floating_window_projection_bundle,
    );

    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].window_id, window_id);
    assert_eq!(targets[0].title, "Native Preview");
    let expected = geometry.floating_window_frame(&targets[0].window_id);
    assert_eq!(
        targets[0].bounds,
        [expected.x, expected.y, expected.width, expected.height]
    );
}
