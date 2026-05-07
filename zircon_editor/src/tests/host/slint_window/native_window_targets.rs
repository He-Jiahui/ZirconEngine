use crate::ui::host::NativeWindowHostState;
use crate::ui::slint_host::callback_dispatch::BuiltinFloatingWindowSourceTemplateBridge;
use crate::ui::slint_host::floating_window_projection::{
    build_floating_window_projection_bundle, resolve_floating_window_projection_base_outer_frame,
    resolve_floating_window_projection_shared_source,
};
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime_interface::ui::layout::UiSize;

use super::support::floating_preview_fixture;

#[test]
fn native_floating_window_targets_fall_back_to_shared_projection_when_host_bounds_are_empty() {
    let window_id = MainPageId::new("window:native-preview");
    let fixture = floating_preview_fixture(&window_id);
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let source_bridge = BuiltinFloatingWindowSourceTemplateBridge::new(UiSize::new(1440.0, 900.0))
        .expect("floating-window source template should build");
    let shared_source =
        resolve_floating_window_projection_shared_source(&source_bridge.source_frames());

    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        shared_source,
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
    let expected = resolve_floating_window_projection_base_outer_frame(
        &model.floating_windows[0],
        0,
        shared_source,
    );
    assert_eq!(
        targets[0].bounds,
        [expected.x, expected.y, expected.width, expected.height]
    );
}
