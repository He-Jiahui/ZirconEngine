use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::icons::{push_audio_icon, push_cube_icon, push_player_start_icon};
use super::kind::{is_unavailable_tree_row_state, tree_icon_kind, TreeIconKind};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

const TREE_OBJECT_BLUE: [u8; 4] = [82, 148, 240, 255];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_object_icon_glyph(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    state: UiPainterResolvedState,
    opacity: f32,
) {
    match tree_icon_kind(node) {
        TreeIconKind::Audio => push_audio_icon(commands, rect, clip, order, color, opacity),
        TreeIconKind::PlayerStart => {
            let icon_color = if is_unavailable_tree_row_state(state) {
                color
            } else {
                TREE_OBJECT_BLUE
            };
            push_player_start_icon(commands, rect, clip, order, icon_color, opacity)
        }
        TreeIconKind::Cube => push_cube_icon(commands, rect, clip, order, color, opacity),
    }
}
