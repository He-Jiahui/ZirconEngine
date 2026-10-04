use super::*;
use crate::core::commands::EditorCommandRegistry;
use crate::ui::workbench::autolayout::{
    compute_workbench_shell_geometry, ShellRegionId, ShellSizePx, WorkbenchChromeMetrics,
};
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;

#[test]
fn authoritative_undersized_solver_frames_do_not_revive_zero_priority_bands() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&EditorCommandRegistry::default_workbench(), &chrome);
    let surface = ShellSizePx::new(900.0, 100.0);
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        surface,
        1.0,
        &WorkbenchChromeMetrics::default(),
        None,
    );
    assert_eq!(geometry.status_bar_frame.height, 0.0);
    assert_eq!(geometry.region_frame(ShellRegionId::Bottom).height, 0.0);

    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.layout =
        crate::ui::retained_host::host_contract::data::HostWindowLayoutData {
            authoritative: true,
            center_band_frame: rect(geometry.center_band_frame),
            status_bar_frame: rect(geometry.status_bar_frame),
            left_region_frame: rect(geometry.region_frame(ShellRegionId::Left)),
            document_region_frame: rect(geometry.region_frame(ShellRegionId::Document)),
            right_region_frame: rect(geometry.region_frame(ShellRegionId::Right)),
            bottom_region_frame: rect(geometry.region_frame(ShellRegionId::Bottom)),
            viewport_content_frame: rect(geometry.viewport_content_frame),
            ..Default::default()
        };

    let roots = resolve_root_frames(surface.width as u32, surface.height as u32, &presentation);
    assert_eq!(roots.status_bar.height, 0.0);
    assert_eq!(roots.bottom_region.height, 0.0);
    for frame in [
        roots.center_band,
        roots.status_bar,
        roots.left_region,
        roots.document_region,
        roots.right_region,
        roots.bottom_region,
        roots.viewport_region,
    ] {
        assert!(frame.x >= 0.0 && frame.y >= 0.0);
        assert!(frame.x + frame.width <= surface.width + f32::EPSILON);
        assert!(frame.y + frame.height <= surface.height + f32::EPSILON);
    }
}

fn rect(frame: crate::ui::workbench::autolayout::ShellFrame) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}
