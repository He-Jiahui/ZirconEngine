mod activity_rail_buttons;
mod clickable_frames;
mod drawer_tabs;
mod frame_math;
mod hit_samples;
mod layout;
mod pane_frames;
mod pane_profile_controls;
mod resize_splitters;
mod tabs;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use pane_frames::collect_surface_frame_controls;

use super::super::data::HostWindowPresentationData;
use super::super::presenter::HostPresenterBackend;
use super::{UiProfileGeometry, UiProfileSize};
use crate::ui::retained_host::primitives::PhysicalSize;
use activity_rail_buttons::collect_activity_rail_profile_buttons;
use clickable_frames::collect_clickable_frames;
use drawer_tabs::collect_drawer_tabs;
use frame_math::visible_profile_frame;
use hit_samples::collect_hit_samples;
use layout::profile_layout;
use pane_profile_controls::collect_pane_profile_controls;
use resize_splitters::collect_resize_splitters;
use tabs::{collect_document_tabs, collect_host_page_tabs};

impl UiProfileGeometry {
    pub(in crate::ui::retained_host::host_contract) fn from_presentation(
        presentation: &HostWindowPresentationData,
        size: &PhysicalSize,
        backend: HostPresenterBackend,
    ) -> Self {
        let scene = &presentation.host_scene_data;
        let resize_splitters = collect_resize_splitters(&scene.resize_layer);
        let document_tabs = collect_document_tabs(&scene.document_dock);
        let drawer_tabs = collect_drawer_tabs(scene);
        let host_page_tabs = collect_host_page_tabs(&scene.page_chrome.tab_frames);
        let activity_rail_buttons = collect_activity_rail_profile_buttons(scene);
        let pane_controls = collect_pane_profile_controls(scene);
        let clickable_frames = collect_clickable_frames(
            &resize_splitters,
            &document_tabs,
            &drawer_tabs,
            &host_page_tabs,
            &activity_rail_buttons,
            &pane_controls.viewport_toolbar_controls,
            &pane_controls.template_controls,
        );
        let hit_samples = collect_hit_samples(&clickable_frames, presentation);

        Self {
            schema_version: 1,
            presenter_backend: backend.label(),
            window_client_size: UiProfileSize {
                width: size.width,
                height: size.height,
            },
            layout: profile_layout(&scene.layout),
            resize_splitters,
            document_tabs,
            drawer_tabs,
            host_page_tabs,
            activity_rail_buttons,
            viewport_frame: visible_profile_frame(&presentation.host_layout.viewport_content_frame),
            viewport_toolbar_controls: pane_controls.viewport_toolbar_controls,
            template_controls: pane_controls.template_controls,
            clickable_frames,
            hit_samples,
        }
    }
}
