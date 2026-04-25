use crate::tests::editor_event::support::env_lock;
use crate::ui::slint_host::callback_dispatch::BuiltinHostWindowTemplateBridge;
use crate::ui::slint_host::drawer_header_pointer::build_host_drawer_header_pointer_layout;
use crate::ui::workbench::autolayout::{
    ShellFrame, ShellRegionId, WorkbenchChromeMetrics, WorkbenchShellGeometry,
};
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime::ui::layout::{UiFrame, UiSize};

#[test]
fn shared_drawer_header_pointer_layout_prefers_shared_root_projection_for_visible_drawer_regions() {
    let _guard = env_lock().lock().unwrap();

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let metrics = WorkbenchChromeMetrics::default();
    let mut bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    bridge
        .recompute_layout_with_workbench_model(UiSize::new(1280.0, 720.0), &model, &metrics)
        .expect("builtin workbench template bridge should recompute visible drawer frames");
    let root_frames = bridge.root_shell_frames();
    let left_geometry = UiFrame::new(180.0, 140.0, 180.0, 519.0);
    let bottom_geometry = UiFrame::new(52.0, 704.0, 777.0, 120.0);
    let geometry = WorkbenchShellGeometry {
        region_frames: [
            (
                ShellRegionId::Left,
                ShellFrame::new(
                    left_geometry.x,
                    left_geometry.y,
                    left_geometry.width,
                    left_geometry.height,
                ),
            ),
            (
                ShellRegionId::Document,
                ShellFrame::new(493.0, 140.0, 531.0, 440.0),
            ),
            (
                ShellRegionId::Right,
                ShellFrame::new(1025.0, 140.0, 255.0, 440.0),
            ),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(
                    bottom_geometry.x,
                    bottom_geometry.y,
                    bottom_geometry.width,
                    bottom_geometry.height,
                ),
            ),
        ]
        .into_iter()
        .collect(),
        ..WorkbenchShellGeometry::default()
    };

    let layout =
        build_host_drawer_header_pointer_layout(&model, &geometry, &metrics, Some(&root_frames));

    let left_surface = layout
        .surfaces
        .iter()
        .find(|surface| surface.key == "left")
        .expect("left drawer surface should exist");
    assert_eq!(
        left_surface.strip_frame,
        root_frames
            .left_drawer_header_frame
            .expect("shared left drawer header frame should exist")
    );

    let bottom_surface = layout
        .surfaces
        .iter()
        .find(|surface| surface.key == "bottom")
        .expect("bottom drawer surface should exist");
    assert_eq!(
        bottom_surface.strip_frame,
        root_frames
            .bottom_drawer_header_frame
            .expect("shared bottom drawer header frame should exist")
    );
}
