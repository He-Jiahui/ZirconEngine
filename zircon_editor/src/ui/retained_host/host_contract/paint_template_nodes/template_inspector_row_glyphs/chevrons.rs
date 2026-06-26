use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::segments::push_inspector_segments;

const INSPECTOR_DROPDOWN_ICON: &str = "zircon_editor_shell/toolbar/dropdown.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_inspector_down_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    if push_icon_asset_pixels(
        commands,
        INSPECTOR_DROPDOWN_ICON,
        rect,
        clip,
        order,
        Some(color),
        opacity,
    ) {
        return;
    }
    let parts = if rect.width >= 14.0 && rect.height >= 14.0 {
        let block = 3.0;
        let center_x = rect.x + rect.width * 0.5;
        let center_y = rect.y + rect.height * 0.5;
        [
            FrameRect {
                x: center_x - block * 1.5,
                y: center_y - block,
                width: block,
                height: block,
            },
            FrameRect {
                x: center_x - block * 0.5,
                y: center_y,
                width: block,
                height: block,
            },
            FrameRect {
                x: center_x + block * 0.5,
                y: center_y - block,
                width: block,
                height: block,
            },
        ]
    } else {
        [
            FrameRect {
                x: rect.x + 2.0,
                y: rect.y + 3.0,
                width: 2.0,
                height: 2.0,
            },
            FrameRect {
                x: rect.x + 4.0,
                y: rect.y + 5.0,
                width: 2.0,
                height: 2.0,
            },
            FrameRect {
                x: rect.x + 6.0,
                y: rect.y + 3.0,
                width: 2.0,
                height: 2.0,
            },
        ]
    };
    push_inspector_segments(commands, clip, order, color, opacity, &parts, 0.0);
}
