use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand, tree::UiTemplateNodeMetadata,
};

use super::{
    commands::{push_label, quad_command},
    geometry::{centered_square, label_rect_after_mark, leading_mark_rect},
    state::SelectionRenderState,
    style::{SelectionVisual, radio_background, radio_border, radio_dot},
};

#[allow(clippy::too_many_arguments)]
pub(super) fn radio_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SelectionRenderState,
    visual: &SelectionVisual,
    frame: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mark = leading_mark_rect(frame, visual);
    let mut commands = vec![quad_command(
        node_id,
        mark,
        clip,
        z.saturating_add(1),
        radio_background(state, visual),
        Some(radio_border(state, visual)),
        visual.border_width,
        mark.height * 0.5,
        state,
        opacity,
    )];
    if state.active() {
        commands.push(quad_command(
            node_id,
            centered_square(mark, visual.radio_dot_size),
            clip,
            z.saturating_add(2),
            radio_dot(state, visual),
            None,
            0.0,
            visual.radio_dot_size * 0.5,
            state,
            opacity,
        ));
    }
    push_label(
        &mut commands,
        node_id,
        metadata,
        label_rect_after_mark(frame, mark, visual),
        clip,
        z.saturating_add(4),
        state,
        visual,
        opacity,
    );
    commands
}
