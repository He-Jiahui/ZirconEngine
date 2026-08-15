use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};
use crate::ui::retained_host::host_contract::profiling_artifacts::UiProfileNamedFrame;
use crate::ui::retained_host::welcome_recent_geometry::welcome_recent_viewport;
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::frame_math::{is_visible_frame, push_named_frame};
use super::surface_frame::collect_surface_frame_controls;
use super::template_nodes::collect_template_node_controls;

pub(in crate::ui::retained_host::host_contract) fn collect_pane_profile_frames(
    surface: &str,
    pane: &PaneData,
    content: &FrameRect,
    viewport_toolbar_controls: &mut Vec<UiProfileNamedFrame>,
    template_controls: &mut Vec<UiProfileNamedFrame>,
    welcome_recent_frames: &mut Vec<UiProfileNamedFrame>,
) {
    if !is_visible_frame(content) {
        return;
    }
    let mut body = content.clone();
    if matches!(pane.kind.as_str(), "Scene" | "Game") && pane.show_toolbar {
        let toolbar_height = 28.0_f32.min(content.height);
        let toolbar = FrameRect {
            x: content.x,
            y: content.y,
            width: content.width,
            height: toolbar_height,
        };
        collect_surface_frame_controls(
            "viewport_toolbar_control",
            surface,
            &toolbar,
            pane.viewport.toolbar_surface_frame.as_deref(),
            viewport_toolbar_controls,
        );
        body.y += toolbar_height;
        body.height = (body.height - toolbar_height).max(0.0);
    }
    if pane.kind.as_str() == "Welcome" {
        let viewport = welcome_recent_viewport(UiSize::new(body.width, body.height));
        push_named_frame(
            welcome_recent_frames,
            "welcome.recent.viewport",
            "welcome_recent_viewport",
            surface,
            FrameRect {
                x: body.x + viewport.x,
                y: body.y + viewport.y,
                width: viewport.width,
                height: viewport.height,
            },
            Some(body.clone()),
        );
    }
    collect_template_node_controls(surface, pane, &body, template_controls);
}
