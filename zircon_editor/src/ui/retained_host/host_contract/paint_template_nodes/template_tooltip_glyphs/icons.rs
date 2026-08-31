use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::template_tooltips::layout::frame_is_within;

const TOOLTIP_INFO_ASSET: &str = "zircon_editor_shell/status/info.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tooltip_info_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    icon_size: f32,
    color: [u8; 4],
    opacity: f32,
) {
    if !icon_size.is_finite() || icon_size <= 0.0 {
        return;
    }
    let y = if node.layout_content_offset_y > 0.0 {
        rect.y + node.layout_content_offset_y
    } else {
        rect.y + rect.height - icon_size
    };
    let icon = FrameRect {
        x: rect.x + (rect.width - icon_size).max(0.0) * 0.5,
        y,
        width: icon_size,
        height: icon_size,
    };
    if !frame_is_within(rect, &icon) {
        return;
    }
    push_icon_asset_pixels(
        commands,
        TOOLTIP_INFO_ASSET,
        &icon,
        clip,
        order,
        Some(color),
        opacity,
    );
}
