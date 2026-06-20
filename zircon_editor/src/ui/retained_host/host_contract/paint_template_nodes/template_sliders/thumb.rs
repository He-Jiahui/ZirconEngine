use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchSliderStyle;
use super::super::template_slider_geometry::{
    centered_rect, slider_thumb_size, SLIDER_THUMB_HALO_SIZE,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_slider_thumb(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    style: &WorkbenchSliderStyle,
    track_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    percent: f32,
    opacity: f32,
) {
    let center_x = track_rect.x + track_rect.width * percent;
    let center_y = track_rect.y + track_rect.height * 0.5;
    let thumb_size = slider_thumb_size(node);
    if let Some(halo_color) = style.thumb_halo {
        commands.push(HostPaintCommand::quad(
            centered_rect(center_x, center_y, SLIDER_THUMB_HALO_SIZE),
            Some(clip.clone()),
            order,
            Some(halo_color),
            None,
            0.0,
            SLIDER_THUMB_HALO_SIZE * 0.5,
            opacity,
        ));
    }
    commands.push(HostPaintCommand::quad(
        centered_rect(center_x, center_y, thumb_size),
        Some(clip.clone()),
        order + 1,
        Some(style.thumb),
        Some(style.thumb_outline),
        1.0,
        thumb_size * 0.5,
        opacity,
    ));
}
