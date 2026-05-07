use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::slint_host::activity_rail_pointer::build_host_activity_rail_pointer_layout;
use crate::ui::slint_host::callback_dispatch::BuiltinHostWindowTemplateBridge;
use crate::ui::workbench::autolayout::{
    compute_workbench_shell_geometry, ShellFrame, ShellRegionId, ShellSizePx,
    WorkbenchChromeMetrics, WorkbenchShellGeometry,
};
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

#[test]
fn shared_activity_rail_pointer_layout_prefers_shared_root_projection_when_left_region_geometry_is_stale(
) {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_activity_rail_pointer_root_projection");
    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let mut geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &harness.runtime.current_layout(),
        &harness.runtime.descriptors(),
        ShellSizePx::new(1280.0, 720.0),
        &WorkbenchChromeMetrics::default(),
        None,
    );
    geometry
        .region_frames
        .insert(ShellRegionId::Left, ShellFrame::default());
    geometry
        .region_frames
        .insert(ShellRegionId::Right, ShellFrame::default());
    geometry
        .region_frames
        .insert(ShellRegionId::Bottom, ShellFrame::default());

    let root_frames = template_bridge.root_shell_frames();
    let layout = build_host_activity_rail_pointer_layout(
        &model,
        &WorkbenchChromeMetrics::default(),
        Some(&root_frames),
    );

    assert_eq!(
        layout.left_strip_frame,
        root_frames.activity_rail_frame.unwrap()
    );
    assert_eq!(layout.right_strip_frame, UiFrame::default());
}

#[test]
fn shared_activity_rail_pointer_layout_prefers_shared_visible_drawer_regions_when_cross_axis_geometry_is_stale(
) {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let metrics = WorkbenchChromeMetrics::default();
    let mut template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    template_bridge
        .recompute_layout_with_workbench_model(UiSize::new(1280.0, 720.0), &model, &metrics)
        .expect("builtin workbench template bridge should recompute visible drawer frames");
    let root_frames = template_bridge.root_shell_frames();
    let left_geometry = ShellFrame::new(180.0, 140.0, 312.0, 519.0);
    let right_geometry = ShellFrame::new(1024.0, 168.0, 256.0, 401.0);
    let bottom_geometry = ShellFrame::new(48.0, 704.0, 777.0, 180.0);
    let left_shared_frame = root_frames
        .left_drawer_shell_frame
        .expect("shared left drawer shell frame should exist");
    let right_shared_frame = root_frames
        .right_drawer_shell_frame
        .expect("shared right drawer shell frame should exist");
    let _stale_geometry = WorkbenchShellGeometry {
        region_frames: [
            (ShellRegionId::Left, left_geometry),
            (
                ShellRegionId::Document,
                ShellFrame::new(493.0, 140.0, 531.0, 440.0),
            ),
            (ShellRegionId::Right, right_geometry),
            (ShellRegionId::Bottom, bottom_geometry),
        ]
        .into_iter()
        .collect(),
        ..WorkbenchShellGeometry::default()
    };

    let layout = build_host_activity_rail_pointer_layout(&model, &metrics, Some(&root_frames));

    assert_eq!(
        layout.left_strip_frame,
        UiFrame::new(
            left_shared_frame.x,
            left_shared_frame.y,
            metrics.rail_width.min(left_shared_frame.width.max(0.0)),
            left_shared_frame.height.max(0.0),
        )
    );
    assert_eq!(
        layout.right_strip_frame,
        UiFrame::new(
            (right_shared_frame.x + right_shared_frame.width - metrics.rail_width)
                .max(right_shared_frame.x),
            right_shared_frame.y,
            metrics.rail_width.min(right_shared_frame.width.max(0.0)),
            right_shared_frame.height.max(0.0),
        )
    );
}
