use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;

const TREE_DISCLOSURE_DOWN_ICON: &str = "zircon_editor_shell/toolbar/dropdown.svg";
const TREE_DISCLOSURE_RIGHT_ICON: &str = "zircon_editor_shell/toolbar/chevron-right.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_disclosure_glyph(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let asset = if node.expanded {
        TREE_DISCLOSURE_DOWN_ICON
    } else {
        TREE_DISCLOSURE_RIGHT_ICON
    };
    push_icon_asset_pixels(commands, asset, rect, clip, order, Some(color), opacity);
}
