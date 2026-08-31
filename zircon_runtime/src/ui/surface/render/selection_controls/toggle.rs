use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand, tree::UiTemplateNodeMetadata,
};

use super::{
    commands::{push_label, quad_command},
    geometry::{toggle_label_rect, toggle_thumb_rect, toggle_track_rect},
    state::SelectionRenderState,
    style::{SelectionVisual, toggle_border, toggle_thumb, toggle_track},
};

#[allow(clippy::too_many_arguments)]
pub(super) fn toggle_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SelectionRenderState,
    visual: &SelectionVisual,
    frame: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let track = toggle_track_rect(frame, visual);
    let mut commands = Vec::new();
    push_label(
        &mut commands,
        node_id,
        metadata,
        toggle_label_rect(frame, track, visual),
        clip,
        z.saturating_add(3),
        state,
        visual,
        opacity,
    );
    commands.push(quad_command(
        node_id,
        track,
        clip,
        z.saturating_add(1),
        toggle_track(state, visual),
        Some(toggle_border(state, visual)),
        visual.border_width,
        track.height * 0.5,
        state,
        opacity,
    ));
    let thumb = toggle_thumb_rect(state, track, visual);
    commands.push(quad_command(
        node_id,
        thumb,
        clip,
        z.saturating_add(2),
        toggle_thumb(state, visual),
        None,
        0.0,
        thumb.height * 0.5,
        state,
        opacity,
    ));
    commands
}
