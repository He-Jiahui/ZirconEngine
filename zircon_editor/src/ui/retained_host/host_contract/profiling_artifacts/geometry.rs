mod activity_rail_buttons;
mod clickable_frames;
mod drawer_tabs;
mod frame_math;
mod hit_samples;
mod layout;
mod native_hierarchy;
mod pane_frames;
mod pane_profile_controls;
mod resize_splitters;
mod rounded_shapes;
mod tabs;
mod text_runs;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use pane_frames::collect_surface_frame_controls;

use super::super::data::HostWindowPresentationData;
use super::super::presenter::HostPresenterBackend;
use super::{UiProfileGeometry, UiProfileLogicalSize, UiProfileSize};
use crate::ui::retained_host::primitives::PhysicalSize;
use activity_rail_buttons::collect_activity_rail_profile_buttons;
use clickable_frames::collect_clickable_frames;
use drawer_tabs::collect_drawer_tabs;
use frame_math::visible_profile_frame;
use hit_samples::collect_hit_samples;
use layout::profile_layout;
use native_hierarchy::collect_native_hierarchy_profiles;
use pane_profile_controls::collect_pane_profile_controls;
use resize_splitters::collect_resize_splitters;
use rounded_shapes::collect_rounded_shapes;
use tabs::{collect_document_tabs, collect_host_page_tabs};
use text_runs::collect_text_runs;

impl UiProfileGeometry {
    pub(in crate::ui::retained_host::host_contract) fn from_presentation(
        presentation: &HostWindowPresentationData,
        size: &PhysicalSize,
        winit_scale_factor: f32,
        backend: HostPresenterBackend,
    ) -> Self {
        Self::from_presentation_with_sequence(presentation, size, winit_scale_factor, backend, 0)
    }

    pub(in crate::ui::retained_host::host_contract) fn from_presentation_with_sequence(
        presentation: &HostWindowPresentationData,
        size: &PhysicalSize,
        winit_scale_factor: f32,
        backend: HostPresenterBackend,
        capture_sequence: u64,
    ) -> Self {
        let scale_factor = if winit_scale_factor.is_finite() && winit_scale_factor > 0.0 {
            f64::from(winit_scale_factor)
        } else {
            1.0
        };
        let logical_size = UiProfileLogicalSize {
            width: f64::from(size.width) / scale_factor,
            height: f64::from(size.height) / scale_factor,
        };
        let scene = &presentation.host_scene_data;
        let resize_splitters = collect_resize_splitters(&scene.resize_layer);
        let document_tabs = collect_document_tabs(&scene.document_dock);
        let drawer_tabs = collect_drawer_tabs(scene);
        let host_page_tabs = collect_host_page_tabs(&scene.page_chrome.tab_frames);
        let activity_rail_buttons = collect_activity_rail_profile_buttons(scene);
        let pane_controls = collect_pane_profile_controls(presentation);
        let native_hierarchy = collect_native_hierarchy_profiles(presentation);
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
            schema_version: 4,
            presenter_backend: backend.label(),
            capture_sequence,
            window_client_size: UiProfileSize {
                width: size.width,
                height: size.height,
            },
            winit_scale_factor,
            window_client_logical_size: logical_size,
            layout: profile_layout(&scene.layout),
            resize_splitters,
            document_tabs,
            drawer_tabs,
            host_page_tabs,
            activity_rail_buttons,
            viewport_frame: visible_profile_frame(&presentation.host_layout.viewport_content_frame),
            viewport_toolbar_controls: pane_controls.viewport_toolbar_controls,
            template_controls: pane_controls.template_controls,
            welcome_recent_frame: pane_controls.welcome_recent_frame,
            asset_browser_content_frame: pane_controls.asset_browser_content_frame,
            clickable_frames,
            hit_samples,
            rounded_shapes: Vec::new(),
            text_runs: Vec::new(),
            submitted_gpu_text: None,
            native_hierarchy,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn from_presentation_with_stream(
        presentation: &HostWindowPresentationData,
        size: &PhysicalSize,
        winit_scale_factor: f32,
        backend: HostPresenterBackend,
        stream: &super::super::chrome_command_stream::ChromeCommandStream,
    ) -> Self {
        Self::from_presentation_with_stream_sequence(
            presentation,
            size,
            winit_scale_factor,
            backend,
            stream,
            0,
        )
    }

    pub(in crate::ui::retained_host::host_contract) fn from_presentation_with_stream_sequence(
        presentation: &HostWindowPresentationData,
        size: &PhysicalSize,
        winit_scale_factor: f32,
        backend: HostPresenterBackend,
        stream: &super::super::chrome_command_stream::ChromeCommandStream,
        capture_sequence: u64,
    ) -> Self {
        Self {
            rounded_shapes: collect_rounded_shapes(stream),
            text_runs: collect_text_runs(stream),
            ..Self::from_presentation_with_sequence(
                presentation,
                size,
                winit_scale_factor,
                backend,
                capture_sequence,
            )
        }
    }
}
