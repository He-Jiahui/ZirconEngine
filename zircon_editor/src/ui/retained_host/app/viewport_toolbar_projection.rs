use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::callback_dispatch;
use crate::ui::retained_host::{PaneData, UiHostWindow};
use zircon_runtime_interface::ui::layout::UiSize;

mod hit_controls;

use hit_controls::viewport_toolbar_hit_control_id;

pub(super) fn attach_viewport_toolbar_surface_frames_to_ui(
    ui: &UiHostWindow,
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    document_viewport_toolbar_width: Option<f32>,
) {
    let mut presentation = ui.get_host_presentation();
    let document_surface_key = presentation
        .host_scene_data
        .document_dock
        .surface_key
        .to_string();
    let document_toolbar_width = document_viewport_toolbar_width
        .filter(|width| *width > f32::EPSILON)
        .unwrap_or_else(|| {
            presentation
                .host_scene_data
                .document_dock
                .content_frame
                .width
                .max(1.0)
        });
    let document_size = UiSize::new(document_toolbar_width.max(1.0), 28.0);
    attach_viewport_toolbar_surface_frame_to_pane(
        viewport_toolbar_bridge,
        document_surface_key,
        document_size,
        &mut presentation.host_scene_data.document_dock.pane,
    );

    let left_surface_key = presentation
        .host_scene_data
        .left_dock
        .surface_key
        .to_string();
    let left_size = UiSize::new(
        presentation
            .host_scene_data
            .left_dock
            .content_frame
            .width
            .max(1.0),
        28.0,
    );
    attach_viewport_toolbar_surface_frame_to_pane(
        viewport_toolbar_bridge,
        left_surface_key,
        left_size,
        &mut presentation.host_scene_data.left_dock.pane,
    );

    let right_surface_key = presentation
        .host_scene_data
        .right_dock
        .surface_key
        .to_string();
    let right_size = UiSize::new(
        presentation
            .host_scene_data
            .right_dock
            .content_frame
            .width
            .max(1.0),
        28.0,
    );
    attach_viewport_toolbar_surface_frame_to_pane(
        viewport_toolbar_bridge,
        right_surface_key,
        right_size,
        &mut presentation.host_scene_data.right_dock.pane,
    );

    let bottom_surface_key = presentation
        .host_scene_data
        .bottom_dock
        .surface_key
        .to_string();
    let bottom_size = UiSize::new(
        presentation
            .host_scene_data
            .bottom_dock
            .content_frame
            .width
            .max(1.0),
        28.0,
    );
    attach_viewport_toolbar_surface_frame_to_pane(
        viewport_toolbar_bridge,
        bottom_surface_key,
        bottom_size,
        &mut presentation.host_scene_data.bottom_dock.pane,
    );

    let mut floating_windows = Vec::new();
    for row in 0..presentation
        .host_scene_data
        .floating_layer
        .floating_windows
        .row_count()
    {
        let Some(mut window) = presentation
            .host_scene_data
            .floating_layer
            .floating_windows
            .row_data(row)
        else {
            continue;
        };
        attach_viewport_toolbar_surface_frame_to_pane(
            viewport_toolbar_bridge,
            window.window_id.to_string(),
            UiSize::new((window.frame.width - 2.0).max(1.0), 28.0),
            &mut window.active_pane,
        );
        floating_windows.push(window);
    }
    presentation.host_scene_data.floating_layer.floating_windows = model_rc(floating_windows);
    ui.set_host_presentation(presentation);
}

fn attach_viewport_toolbar_surface_frame_to_pane(
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
