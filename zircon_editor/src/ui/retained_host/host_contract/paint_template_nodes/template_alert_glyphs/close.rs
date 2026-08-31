use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;

const CLOSE_MARK_ASSET: &str = "ionicons/close-outline.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_close_mark(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_icon_asset_pixels(
        commands,
        CLOSE_MARK_ASSET,
        rect,
        clip,
        order,
        Some(color),
        opacity,
    );
}
