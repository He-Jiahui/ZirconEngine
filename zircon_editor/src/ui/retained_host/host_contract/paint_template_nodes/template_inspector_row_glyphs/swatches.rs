use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;

const INSPECTOR_SWATCH_SIZE: f32 = 12.0;
const MATERIAL_SWATCH: [u8; 4] = [34, 176, 192, 255];
const MATERIAL_SWATCH_BORDER: [u8; 4] = [21, 95, 105, 255];
const INSPECTOR_MATERIAL_ICON: &str = "zircon_editor_shell/inspector/material.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_inspector_swatch(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let swatch = FrameRect {
        x: rect.x + (rect.width - INSPECTOR_SWATCH_SIZE).max(0.0) * 0.5,
        y: rect.y + (rect.height - INSPECTOR_SWATCH_SIZE).max(0.0) * 0.5,
        width: INSPECTOR_SWATCH_SIZE,
        height: INSPECTOR_SWATCH_SIZE,
    };
    if push_icon_asset_pixels(
        commands,
        INSPECTOR_MATERIAL_ICON,
        &swatch,
        clip,
        order,
        None,
        opacity,
    ) {
        return;
    }
    commands.push(HostPaintCommand::quad(
        swatch,
        Some(clip.clone()),
        order,
        Some(MATERIAL_SWATCH),
        Some(MATERIAL_SWATCH_BORDER),
        1.0,
        INSPECTOR_SWATCH_SIZE * 0.5,
        opacity,
    ));
}
