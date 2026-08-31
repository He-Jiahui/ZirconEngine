use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand, tree::UiTemplateNodeMetadata,
};

use super::{
    commands::{push_label, quad_command},
    geometry::{label_rect_after_mark, leading_mark_rect},
    state::SelectionRenderState,
    style::{SelectionVisual, checkbox_background, checkbox_border},
};

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
    let mut commands = vec![quad_command(
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
    )];
    if state.active() {
        commands.extend(checkbox_tick_commands(
            node_id,
            mark,
            clip,
            z.saturating_add(2),
            state,
            visual,
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

fn checkbox_tick_commands(
    node_id: UiNodeId,
    mark: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    state: &SelectionRenderState,
    visual: &SelectionVisual,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let unit = mark.width * (3.0 / 16.0);
    [
        (3.0, 7.0, 3.0, 3.0),
        (5.0, 9.0, 3.0, 3.0),
        (8.0, 4.0, 3.0, 8.0),
    ]
    .into_iter()
    .map(|(x, y, w, h)| {
        quad_command(
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
        )
    })
    .collect()
}
