use zircon_runtime_interface::ui::event_ui::UiTreeId;

use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::workbench::layout::MainPageId;

use super::{native_floating_presentation_matches, FrameRect, NativeFloatingWindowTarget};

#[test]
fn unchanged_native_target_does_not_require_a_presentation_rebuild() {
    let target = NativeFloatingWindowTarget {
        window_id: MainPageId("window-1".to_owned()),
        title: "Tools".to_owned(),
        bounds: [12.0, 24.0, 640.0, 480.0],
        surface_tree_id: UiTreeId("tree-1".to_owned()),
    };
    let bounds = FrameRect {
        x: 12.0,
        y: 24.0,
        width: 640.0,
        height: 480.0,
    };
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_shell.native_floating_window_mode = true;
    presentation.host_shell.native_floating_window_id = target.window_id.0.clone();
    presentation.host_shell.native_surface_tree_id = target.surface_tree_id.0.clone();
    presentation.host_shell.native_window_title = target.title.clone();
    presentation.host_shell.native_window_bounds = bounds.clone();
    presentation
        .native_floating_surface_data
        .native_floating_window_id = target.window_id.0.clone();
    presentation
        .native_floating_surface_data
        .native_surface_tree_id = target.surface_tree_id.0.clone();
    presentation
        .native_floating_surface_data
        .native_window_bounds = bounds.clone();

    assert!(native_floating_presentation_matches(
        &presentation,
        &target,
        &bounds
    ));

    presentation.host_shell.native_window_title = "Changed".to_owned();
    assert!(!native_floating_presentation_matches(
        &presentation,
        &target,
        &bounds
    ));
}
