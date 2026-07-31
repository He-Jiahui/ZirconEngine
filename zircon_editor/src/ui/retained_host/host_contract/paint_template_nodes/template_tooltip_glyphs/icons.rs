use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_tooltips::layout::frame_is_within;

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
    commands.push(HostPaintCommand::quad(
        icon.clone(),
        Some(clip.clone()),
        order,
        None,
        Some(color),
        1.0,
        icon_size * 0.5,
        opacity,
    ));

    let stem_width = icon_size * 0.12;
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: icon.x + (icon.width - stem_width) * 0.5,
            y: icon.y + icon.height * 0.45,
            width: stem_width,
            height: icon.height * 0.33,
        },
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        1.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: icon.x + (icon.width - stem_width) * 0.5,
            y: icon.y + icon.height * 0.25,
            width: stem_width,
            height: stem_width,
        },
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        stem_width * 0.5,
        opacity,
    ));
}
