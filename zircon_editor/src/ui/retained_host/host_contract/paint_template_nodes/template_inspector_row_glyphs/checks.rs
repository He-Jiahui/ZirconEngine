use super::super::super::data::FrameRect;
use super::super::super::paint_theme::PALETTE;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::segments::push_inspector_segments;

const INSPECTOR_CHECK_ICON: &str = "zircon_editor_shell/controls/check.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_inspector_check_tick(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if push_icon_asset_pixels(
        commands,
        INSPECTOR_CHECK_ICON,
        rect,
        clip,
        order,
        Some(PALETTE.shell_background),
        opacity,
    ) {
        return;
    }
    push_inspector_segments(
        commands,
        clip,
        order,
        PALETTE.shell_background,
        opacity,
        &[
            FrameRect {
                x: rect.x + 3.0,
                y: rect.y + 7.0,
                width: 3.0,
                height: 3.0,
            },
            FrameRect {
                x: rect.x + 5.0,
                y: rect.y + 9.0,
                width: 3.0,
                height: 3.0,
            },
            FrameRect {
                x: rect.x + 8.0,
                y: rect.y + 4.0,
                width: 3.0,
                height: 8.0,
            },
        ],
        1.0,
    );
}
