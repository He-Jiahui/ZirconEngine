use super::super::super::data::HostWindowSceneData;
use super::super::UiProfileNamedFrame;
use super::frame_math::translated;
use super::pane_frames::{
    collect_pane_profile_frames, floating_window_content_frame, side_dock_content_frame,
};

pub(in crate::ui::retained_host::host_contract) struct PaneProfileControls {
    pub(in crate::ui::retained_host::host_contract) viewport_toolbar_controls:
        Vec<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) template_controls: Vec<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) welcome_recent_frame:
        Option<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) asset_browser_content_frame:
        Option<UiProfileNamedFrame>,
}

pub(in crate::ui::retained_host::host_contract) fn collect_pane_profile_controls(
    scene: &HostWindowSceneData,
) -> PaneProfileControls {
    let mut viewport_toolbar_controls = Vec::new();
    let mut template_controls = Vec::new();
    let mut welcome_recent_frames = Vec::new();
    let mut asset_browser_content_frames = Vec::new();
    collect_pane_profile_frames(
        "document",
        &scene.document_dock.pane,
        &translated(
            &scene.document_dock.content_frame,
            scene.document_dock.region_frame.x,
            scene.document_dock.region_frame.y,
        ),
        &mut viewport_toolbar_controls,
        &mut template_controls,
        &mut welcome_recent_frames,
        &mut asset_browser_content_frames,
    );
    collect_pane_profile_frames(
        "left",
        &scene.left_dock.pane,
        &side_dock_content_frame(&scene.left_dock),
        &mut viewport_toolbar_controls,
        &mut template_controls,
        &mut welcome_recent_frames,
        &mut asset_browser_content_frames,
    );
    collect_pane_profile_frames(
        "right",
        &scene.right_dock.pane,
        &side_dock_content_frame(&scene.right_dock),
        &mut viewport_toolbar_controls,
        &mut template_controls,
        &mut welcome_recent_frames,
        &mut asset_browser_content_frames,
    );
    collect_pane_profile_frames(
        "bottom",
        &scene.bottom_dock.pane,
        &translated(
            &scene.bottom_dock.content_frame,
            scene.bottom_dock.region_frame.x,
            scene.bottom_dock.region_frame.y,
        ),
        &mut viewport_toolbar_controls,
        &mut template_controls,
        &mut welcome_recent_frames,
        &mut asset_browser_content_frames,
    );
    for row in 0..scene.floating_layer.floating_windows.row_count() {
        if let Some(window) = scene.floating_layer.floating_windows.row_data(row) {
            collect_pane_profile_frames(
                window.window_id.as_str(),
                &window.active_pane,
                &floating_window_content_frame(&window.frame, &window.header_frame),
                &mut viewport_toolbar_controls,
                &mut template_controls,
                &mut welcome_recent_frames,
                &mut asset_browser_content_frames,
            );
        }
    }
    PaneProfileControls {
        viewport_toolbar_controls,
        template_controls,
        welcome_recent_frame: welcome_recent_frames.into_iter().next(),
        asset_browser_content_frame: asset_browser_content_frames.into_iter().next(),
    }
}
