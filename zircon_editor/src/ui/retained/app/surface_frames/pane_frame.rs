use crate::ui::retained_host::{callback_dispatch, PaneData};
use std::sync::Arc;
use zircon_runtime_interface::ui::layout::UiSize;
use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

use super::super::hit_controls::{viewport_toolbar_hit_control_id, viewport_toolbar_hit_route_key};

const VIEWPORT_TOOLBAR_HEIGHT: f32 = 28.0;

pub(super) fn viewport_toolbar_size_for_width(width: f32, scale: f32) -> UiSize {
    UiSize::new(width.max(1.0), VIEWPORT_TOOLBAR_HEIGHT * scale)
}

pub(super) fn next_viewport_toolbar_surface_frame_pane(
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    surface_key: &str,
    toolbar_size: UiSize,
    pane: &PaneData,
) -> Option<PaneData> {
    let next_frame =
        viewport_toolbar_surface_frame(viewport_toolbar_bridge, surface_key, toolbar_size, pane);
    let admission = viewport_toolbar_bridge.play_admission();
    (!same_surface_frame(
        pane.viewport.toolbar_surface_frame.as_ref(),
        next_frame.as_ref(),
    ) || (
        pane.viewport.toolbar_enter_play_enabled,
        pane.viewport.toolbar_exit_play_enabled,
        pane.viewport.toolbar_is_playing,
    ) != admission)
        .then(|| {
            let mut next = pane.clone();
            next.viewport.toolbar_template_nodes = next_frame
                .as_ref()
                .and_then(|_| {
                    viewport_toolbar_bridge
                        .paint_nodes_for_size(toolbar_size)
                        .ok()
                })
                .unwrap_or_default();
            next.viewport.toolbar_surface_frame = next_frame;
            next.viewport.toolbar_surface_key = surface_key.into();
            (
                next.viewport.toolbar_enter_play_enabled,
                next.viewport.toolbar_exit_play_enabled,
                next.viewport.toolbar_is_playing,
            ) = admission;
            next
        })
}

fn same_surface_frame(
    left: Option<&Arc<UiSurfaceFrame>>,
    right: Option<&Arc<UiSurfaceFrame>>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => Arc::ptr_eq(left, right),
        _ => false,
    }
}

pub(super) fn attach_viewport_toolbar_surface_frame_to_pane(
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    surface_key: &str,
    toolbar_size: UiSize,
    pane: &mut PaneData,
) {
    let surface_frame =
        viewport_toolbar_surface_frame(viewport_toolbar_bridge, surface_key, toolbar_size, pane);
    if !same_surface_frame(
        pane.viewport.toolbar_surface_frame.as_ref(),
        surface_frame.as_ref(),
    ) {
        pane.viewport.toolbar_template_nodes = surface_frame
            .as_ref()
            .and_then(|_| {
                viewport_toolbar_bridge
                    .paint_nodes_for_size(toolbar_size)
                    .ok()
            })
            .unwrap_or_default();
    }
    pane.viewport.toolbar_surface_frame = surface_frame;
    pane.viewport.toolbar_surface_key = surface_key.into();
    (
        pane.viewport.toolbar_enter_play_enabled,
        pane.viewport.toolbar_exit_play_enabled,
        pane.viewport.toolbar_is_playing,
    ) = viewport_toolbar_bridge.play_admission();
}

fn viewport_toolbar_surface_frame(
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    surface_key: &str,
    toolbar_size: UiSize,
    pane: &PaneData,
) -> Option<Arc<UiSurfaceFrame>> {
    if !matches!(pane.kind.as_str(), "Scene" | "Game") || !pane.show_toolbar {
        return None;
    }

    let cached_surface_frame = {
        let viewport = &pane.viewport;
        let hit_route_key = viewport_toolbar_hit_route_key(viewport);
        viewport_toolbar_bridge
            .surface_frame_from_cached_layout_for_projection_controls_with_hit_route_key(
                surface_key,
                toolbar_size,
                &hit_route_key,
                |projection_control_id| {
                    Some(viewport_toolbar_hit_control_id(
                        viewport,
                        projection_control_id,
                    ))
                },
            )
    };
    if let Some(surface_frame) = cached_surface_frame {
        return Some(surface_frame);
    }

    if viewport_toolbar_bridge
        .recompute_layout(toolbar_size)
        .is_err()
    {
        return None;
    }

    let surface_frame = {
        let viewport = &pane.viewport;
        let hit_route_key = viewport_toolbar_hit_route_key(viewport);
        viewport_toolbar_bridge.surface_frame_for_projection_controls_with_hit_route_key(
            surface_key,
            toolbar_size,
            &hit_route_key,
            |projection_control_id| {
                Some(viewport_toolbar_hit_control_id(
                    viewport,
                    projection_control_id,
                ))
            },
        )
    };
    Some(surface_frame)
}

#[cfg(test)]
#[path = "tests/pane_frame.rs"]
mod tests;
