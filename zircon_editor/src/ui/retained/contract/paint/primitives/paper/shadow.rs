use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::render_commands::HostPaintCommand;

const PAPER_SHADOW_AMBIENT_ALPHA_SCALE: f32 = 0.27;
const PAPER_SHADOW_PENUMBRA_ALPHA_SCALE: f32 = 0.31;
const PAPER_SHADOW_UMBRA_ALPHA_SCALE: f32 = 0.44;

struct ShadowLayer {
    offset_y: f32,
    grow: f32,
    color: [u8; 4],
}

/// 三层阴影沿用宿主阴影色并早于 Paper 表面提交；调用方负责只在非 outlined 高架状态调用。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_paper_shadow(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    elevation: f32,
    corner_radius: f32,
    opacity: f32,
) {
    for (index, layer) in shadow_layers(elevation).into_iter().enumerate() {
        commands.push(
            HostPaintCommand::quad(
                expanded_offset_rect(rect, layer.offset_y, layer.grow),
                Some(clip.clone()),
                order + index as i32,
                Some(layer.color),
                None,
                0.0,
                (corner_radius + layer.grow).max(0.0),
                opacity,
            )
            .with_box_shadow(
                0.0,
                layer.offset_y,
                0.0,
                layer.grow,
                (corner_radius + layer.grow).max(0.0),
                false,
            ),
        );
    }
}

fn shadow_layers(elevation: f32) -> [ShadowLayer; 3] {
    shadow_layers_from_host(elevation, current_host_palette())
}

fn shadow_layers_from_host(elevation: f32, palette: HostMaterialPalette) -> [ShadowLayer; 3] {
    let elevation = elevation.clamp(1.0, 24.0);
    let offset = elevation.round().max(1.0);
    [
        ShadowLayer {
            offset_y: (elevation / 3.0).round().max(1.0),
            grow: 1.0,
            color: shadow_layer_color(palette.shadow, PAPER_SHADOW_AMBIENT_ALPHA_SCALE),
        },
        ShadowLayer {
            offset_y: offset,
            grow: 0.0,
            color: shadow_layer_color(palette.shadow, PAPER_SHADOW_PENUMBRA_ALPHA_SCALE),
        },
        ShadowLayer {
            offset_y: offset,
            grow: 0.0,
            color: shadow_layer_color(palette.shadow, PAPER_SHADOW_UMBRA_ALPHA_SCALE),
        },
    ]
}

fn shadow_layer_color(base: [u8; 4], alpha_scale: f32) -> [u8; 4] {
    [
        base[0],
        base[1],
        base[2],
        ((base[3] as f32) * alpha_scale)
            .round()
            .clamp(0.0, u8::MAX as f32) as u8,
    ]
}

fn expanded_offset_rect(rect: &FrameRect, offset_y: f32, grow: f32) -> FrameRect {
    FrameRect {
        x: rect.x - grow,
        y: rect.y + offset_y - grow,
        width: rect.width + grow * 2.0,
        height: rect.height + grow * 2.0,
    }
}

#[cfg(test)]
#[path = "tests/shadow.rs"]
mod tests;
