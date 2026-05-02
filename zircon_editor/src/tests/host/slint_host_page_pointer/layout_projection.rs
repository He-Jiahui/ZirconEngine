use crate::tests::editor_event::support::EventRuntimeHarness;
use crate::ui::slint_host::callback_dispatch::{
    BuiltinHostRootShellFrames, BuiltinHostWindowTemplateBridge,
};
use crate::ui::slint_host::host_page_pointer::build_host_page_pointer_layout;
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

#[test]
fn shared_host_page_pointer_layout_prefers_shared_shell_width_over_metric_strip_estimate() {
    let harness = EventRuntimeHarness::new("zircon_slint_host_page_pointer_shared_width");
    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let root_frames = template_bridge.root_shell_frames();
    let layout = build_host_page_pointer_layout(
        &model,
        &WorkbenchChromeMetrics::default(),
        Some(&root_frames),
    );

    assert_eq!(
        layout.strip_frame,
        UiFrame::new(0.0, 26.0, 1280.0, 32.0),
        "shared shell projection should own the root host-page strip width"
    );
}

#[test]
fn shared_host_page_pointer_layout_prefers_shared_host_strip_frame_over_shell_metric_estimate() {
    let harness = EventRuntimeHarness::new("zircon_slint_host_page_pointer_shared_strip");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let layout = build_host_page_pointer_layout(
        &model,
        &WorkbenchChromeMetrics::default(),
        Some(&BuiltinHostRootShellFrames {
            shell_frame: Some(UiFrame::new(32.0, 18.0, 1440.0, 900.0)),
            host_page_strip_frame: Some(UiFrame::new(40.0, 54.0, 1110.0, 28.0)),
            ..Default::default()
        }),
    );

    assert_eq!(
        layout.strip_frame,
        UiFrame::new(40.0, 54.0, 1110.0, 28.0),
        "shared host-page strip projection should outrank the shell-level metric estimate"
    );
}
