use crate::ui::retained_host::{PaneData, callback_dispatch};
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::hit_controls::viewport_toolbar_hit_control_id;

const VIEWPORT_TOOLBAR_HEIGHT: f32 = 28.0;

pub(super) fn viewport_toolbar_size_for_width(width: f32) -> UiSize {
    UiSize::new(width.max(1.0), VIEWPORT_TOOLBAR_HEIGHT)
}

pub(super) fn attach_viewport_toolbar_surface_frame_to_pane(
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    surface_key: String,
    toolbar_size: UiSize,
    pane: &mut PaneData,
) {
    if !matches!(pane.kind.as_str(), "Scene" | "Game") || !pane.show_toolbar {
        pane.viewport.toolbar_surface_frame = None;
        return;
    }

    if viewport_toolbar_bridge
        .recompute_layout(toolbar_size)
        .is_err()
    {
        pane.viewport.toolbar_surface_frame = None;
        return;
    }

    let viewport = pane.viewport.clone();
    pane.viewport.toolbar_surface_frame = Some(
        viewport_toolbar_bridge.surface_frame_for_projection_controls(
            &surface_key,
            toolbar_size,
            |projection_control_id| {
                Some(viewport_toolbar_hit_control_id(
                    &viewport,
                    projection_control_id,
                ))
            },
        ),
    );
}
