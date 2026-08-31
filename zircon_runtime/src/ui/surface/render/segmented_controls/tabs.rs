use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand, tree::UiTemplateNodeMetadata,
};

use super::{
    commands::{quad_command, text_command},
    metadata::tab_label,
    state::SegmentedRenderState,
    style::{SegmentedVisual, selected_underline, tab_background, tab_text_color},
};

#[allow(clippy::too_many_arguments)]
pub(super) fn tab_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
    frame: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mut commands = Vec::new();
    if let Some(background) = tab_background(metadata, state, visual) {
        commands.push(quad_command(
            node_id,
            frame,
            clip,
            z.saturating_add(1),
            background,
            None,
            0.0,
            0.0,
            state,
            opacity,
        ));
    }
    if state.active() {
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                frame.x,
                frame.y + (frame.height - visual.tab_underline_height).max(0.0),
                frame.width,
                visual.tab_underline_height,
            ),
            clip,
            z.saturating_add(3),
            selected_underline(state, visual),
            None,
            0.0,
            0.0,
            state,
            opacity,
        ));
    }
    if let Some(label) = tab_label(metadata) {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                frame.x + visual.tab_text_inset_x,
                frame.y + (frame.height - visual.tab_line_height).max(0.0) * 0.5,
                (frame.width - visual.tab_text_inset_x * 2.0).max(visual.min_frame_extent),
                visual.tab_line_height,
            ),
            clip,
            z.saturating_add(4),
            label,
            tab_text_color(state, visual),
            visual.tab_font_size,
            visual.tab_line_height,
            state,
            opacity,
        ));
    }
    commands
}
