mod authored;

use super::super::super::componentized_workbench_regions::{
    owns_authored_level_viewport, owns_ordinary_panes,
};
use super::super::super::data::HostWindowPresentationData;
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
    presentation: &HostWindowPresentationData,
) -> PaneProfileControls {
    let scene = &presentation.host_scene_data;
    let mut viewport_toolbar_controls = Vec::new();
    let mut template_controls = Vec::new();
    let mut welcome_recent_frames = Vec::new();
    let mut asset_browser_content_frames = Vec::new();
    for document in scene.document_surfaces() {
        if owns_authored_level_viewport(presentation, document) {
            continue;
        }
        let surface = if scene.document_leaves.is_empty() {
            "document"
        } else {
            document.surface_key.as_str()
        };
        collect_pane_profile_frames(
            surface,
            &document.pane,
            &translated(
                &document.content_frame,
                document.region_frame.x,
                document.region_frame.y,
            ),
            &mut viewport_toolbar_controls,
            &mut template_controls,
            &mut welcome_recent_frames,
            &mut asset_browser_content_frames,
        );
    }
    if owns_ordinary_panes(presentation) {
        authored::collect_authored_pane_controls(
            presentation,
            &mut template_controls,
            &mut viewport_toolbar_controls,
        );
    } else {
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
    }
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
