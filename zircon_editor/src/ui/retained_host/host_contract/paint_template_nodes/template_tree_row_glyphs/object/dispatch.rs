use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::kind::{is_unavailable_tree_row_state, tree_icon_kind, TreeIconKind};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

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
    let kind = tree_icon_kind(node);
    let tint = is_unavailable_tree_row_state(state).then_some(color);
    push_icon_asset_pixels(
        commands,
        tree_icon_asset_name(node, kind),
        rect,
        clip,
        order,
        tint,
        opacity,
    );
}

fn tree_icon_asset_name(node: &TemplatePaneNodeData, kind: TreeIconKind) -> &str {
    if !node.icon_name.trim().is_empty() {
        return node.icon_name.as_str();
    }
    match kind {
        TreeIconKind::Audio => "zircon_editor_shell/scene/audio-zone.svg",
        TreeIconKind::PlayerStart => "zircon_editor_shell/scene/player-start.svg",
        TreeIconKind::Cube => "zircon_editor_shell/activity/cube.svg",
    }
}
