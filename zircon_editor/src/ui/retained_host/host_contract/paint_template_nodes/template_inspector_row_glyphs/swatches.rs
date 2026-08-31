use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;

const INSPECTOR_SWATCH_SIZE: f32 = 12.0;
const INSPECTOR_MATERIAL_ICON: &str = "zircon_editor_shell/inspector/material.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_inspector_swatch(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !can_fit_inspector_swatch(rect) {
        return;
    }
    let swatch = FrameRect {
        x: rect.x + (rect.width - INSPECTOR_SWATCH_SIZE).max(0.0) * 0.5,
        y: rect.y + (rect.height - INSPECTOR_SWATCH_SIZE).max(0.0) * 0.5,
        width: INSPECTOR_SWATCH_SIZE,
        height: INSPECTOR_SWATCH_SIZE,
    };
    push_icon_asset_pixels(
        commands,
        INSPECTOR_MATERIAL_ICON,
        &swatch,
        clip,
        order,
        None,
        opacity,
    );
}

fn can_fit_inspector_swatch(rect: &FrameRect) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width >= INSPECTOR_SWATCH_SIZE
        && rect.height >= INSPECTOR_SWATCH_SIZE
}
