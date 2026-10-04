use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand, tree::UiTemplateNodeMetadata,
};

use super::{
    commands::{push_label, quad_command},
    geometry::{label_rect_after_mark, leading_mark_rect},
    state::SelectionRenderState,
    style::{checkbox_background, checkbox_border, SelectionVisual},
};

const CHECKBOX_COMMAND_CAPACITY: usize = 5;

#[allow(clippy::too_many_arguments)]
pub(super) fn checkbox_commands(
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
    let mut commands = Vec::with_capacity(CHECKBOX_COMMAND_CAPACITY);
    commands.push(quad_command(
        node_id,
        mark,
        clip,
        z.saturating_add(1),
        checkbox_background(state, visual),
        Some(checkbox_border(state, visual)),
        visual.border_width,
        visual.mark_radius,
        state,
        opacity,
    ));
    if state.active() {
        push_checkbox_tick_commands(
            &mut commands,
            node_id,
            mark,
            clip,
            z.saturating_add(2),
            state,
            visual,
            opacity,
        );
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

fn push_checkbox_tick_commands(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    mark: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    state: &SelectionRenderState,
    visual: &SelectionVisual,
    opacity: f32,
) {
    let unit = mark.width * (3.0 / 16.0);
    for (x, y, w, h) in [
        (3.0, 7.0, 3.0, 3.0),
        (5.0, 9.0, 3.0, 3.0),
        (8.0, 4.0, 3.0, 8.0),
    ] {
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                mark.x + x * unit / 3.0,
                mark.y + y * unit / 3.0,
                w * unit / 3.0,
                h * unit / 3.0,
            ),
            clip,
            z,
            visual.accent,
            None,
            0.0,
            visual.border_width,
            state,
            opacity,
        ));
    }
}
