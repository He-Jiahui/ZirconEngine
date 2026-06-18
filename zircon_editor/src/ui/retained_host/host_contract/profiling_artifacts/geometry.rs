mod frame_math;
mod hit_samples;
mod pane_frames;
mod tabs;

#[cfg(test)]
pub(super) use pane_frames::collect_surface_frame_controls;

use super::super::data::HostWindowPresentationData;
use super::super::presenter::HostPresenterBackend;
use super::{UiProfileGeometry, UiProfileLayout, UiProfileNamedFrame, UiProfileSize};
use crate::ui::retained_host::primitives::PhysicalSize;
use frame_math::{push_named_frame, translated, visible_profile_frame};
use hit_samples::hit_samples_for_frame;
use pane_frames::{
    collect_activity_rail_buttons, collect_pane_profile_frames, floating_window_content_frame,
    side_dock_content_frame,
};
use tabs::{
    collect_bottom_dock_tabs, collect_document_tabs, collect_floating_window_tabs,
    collect_host_page_tabs, collect_side_dock_tabs,
};

impl UiProfileGeometry {
    pub(super) fn from_presentation(
        presentation: &HostWindowPresentationData,
        size: &PhysicalSize,
        backend: HostPresenterBackend,
    ) -> Self {
        let scene = &presentation.host_scene_data;
        let mut resize_splitters = Vec::new();
        push_named_frame(
            &mut resize_splitters,
            "resize.left_splitter",
            "resize_splitter",
            "left",
            scene.resize_layer.left_splitter_frame.clone(),
            None,
        );
        push_named_frame(
            &mut resize_splitters,
            "resize.right_splitter",
            "resize_splitter",
            "right",
            scene.resize_layer.right_splitter_frame.clone(),
            None,
        );
        push_named_frame(
            &mut resize_splitters,
            "resize.bottom_splitter",
            "resize_splitter",
            "bottom",
            scene.resize_layer.bottom_splitter_frame.clone(),
            None,
        );

        let document_tabs = collect_document_tabs(&scene.document_dock);
        let mut drawer_tabs = Vec::new();
        collect_side_dock_tabs("left", &scene.left_dock, &mut drawer_tabs);
        collect_side_dock_tabs("right", &scene.right_dock, &mut drawer_tabs);
        collect_bottom_dock_tabs("bottom", &scene.bottom_dock, &mut drawer_tabs);
        for row in 0..scene.floating_layer.floating_windows.row_count() {
            if let Some(window) = scene.floating_layer.floating_windows.row_data(row) {
                collect_floating_window_tabs(&window, &mut drawer_tabs);
            }
        }

        let host_page_tabs = collect_host_page_tabs(&scene.page_chrome.tab_frames);
        let mut activity_rail_buttons = Vec::new();
        collect_activity_rail_buttons("left", &scene.left_dock, &mut activity_rail_buttons);
        collect_activity_rail_buttons("right", &scene.right_dock, &mut activity_rail_buttons);

        let mut viewport_toolbar_controls = Vec::new();
        let mut template_controls = Vec::new();
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
        );
        collect_pane_profile_frames(
            "left",
            &scene.left_dock.pane,
            &side_dock_content_frame(&scene.left_dock),
            &mut viewport_toolbar_controls,
            &mut template_controls,
        );
        collect_pane_profile_frames(
            "right",
            &scene.right_dock.pane,
            &side_dock_content_frame(&scene.right_dock),
            &mut viewport_toolbar_controls,
            &mut template_controls,
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
        );
        for row in 0..scene.floating_layer.floating_windows.row_count() {
            if let Some(window) = scene.floating_layer.floating_windows.row_data(row) {
                collect_pane_profile_frames(
                    window.window_id.as_str(),
                    &window.active_pane,
                    &floating_window_content_frame(&window.frame, &window.header_frame),
                    &mut viewport_toolbar_controls,
                    &mut template_controls,
                );
            }
        }

        let mut clickable_frames = Vec::new();
        clickable_frames.extend(resize_splitters.iter().cloned());
        clickable_frames.extend(document_tabs.iter().map(UiProfileNamedFrame::from_tab));
        clickable_frames.extend(drawer_tabs.iter().map(UiProfileNamedFrame::from_tab));
        clickable_frames.extend(host_page_tabs.iter().map(UiProfileNamedFrame::from_tab));
        clickable_frames.extend(activity_rail_buttons.iter().cloned());
        clickable_frames.extend(viewport_toolbar_controls.iter().cloned());
        clickable_frames.extend(template_controls.iter().cloned());

        let hit_samples = clickable_frames
            .iter()
            .flat_map(|frame| hit_samples_for_frame(frame, presentation))
            .collect();

        Self {
            schema_version: 1,
            presenter_backend: backend.label(),
            window_client_size: UiProfileSize {
                width: size.width,
                height: size.height,
            },
            layout: UiProfileLayout {
                center_band: scene.layout.center_band_frame.clone().into(),
                document_region: scene.layout.document_region_frame.clone().into(),
                left_region: scene.layout.left_region_frame.clone().into(),
                right_region: scene.layout.right_region_frame.clone().into(),
                bottom_region: scene.layout.bottom_region_frame.clone().into(),
                status_bar: scene.layout.status_bar_frame.clone().into(),
            },
            resize_splitters,
            document_tabs,
            drawer_tabs,
            host_page_tabs,
            activity_rail_buttons,
            viewport_frame: visible_profile_frame(&presentation.host_layout.viewport_content_frame),
            viewport_toolbar_controls,
            template_controls,
            clickable_frames,
            hit_samples,
        }
    }
}
