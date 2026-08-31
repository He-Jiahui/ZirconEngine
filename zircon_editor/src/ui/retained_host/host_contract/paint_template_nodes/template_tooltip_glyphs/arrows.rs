use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::template_tooltips::layout::frame_is_within;

const TOOLTIP_ARROW_ASSET: &str = "zircon_editor_shell/controls/tooltip-arrow.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tooltip_arrow(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    bubble: &FrameRect,
    clip: &FrameRect,
    order: i32,
    arrow_size: f32,
    fill: [u8; 4],
    border: [u8; 4],
    opacity: f32,
) {
    if !arrow_size.is_finite() || arrow_size < 2.0 {
        return;
    }
    let x = bubble.x + bubble.width * 0.5 - arrow_size * 0.5;
    let y = bubble.y + bubble.height - 1.0;
    let arrow = FrameRect {
        x,
        y,
        width: arrow_size,
        height: arrow_size,
    };
    if !frame_is_within(rect, &arrow) {
        return;
    }
    push_icon_asset_pixels(
        commands,
        TOOLTIP_ARROW_ASSET,
        &arrow,
        clip,
        order,
        Some(border),
        opacity,
    );

    let fill_size = arrow_size - 2.0;
    if fill_size >= 2.0 {
        let fill_rect = FrameRect {
            x: x + 1.0,
            y: y + 1.0,
            width: fill_size,
            height: fill_size,
        };
        push_icon_asset_pixels(
            commands,
            TOOLTIP_ARROW_ASSET,
            &fill_rect,
            clip,
            order + 1,
            Some(fill),
            opacity,
        );
    }
}
