use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::{HostMaterialPalette, current_host_palette};
use super::super::super::render_commands::HostPaintCommand;

const PAPER_SHADOW_AMBIENT_ALPHA_SCALE: f32 = 0.27;
const PAPER_SHADOW_PENUMBRA_ALPHA_SCALE: f32 = 0.31;
const PAPER_SHADOW_UMBRA_ALPHA_SCALE: f32 = 0.44;

struct ShadowLayer {
    offset_y: f32,
    grow: f32,
    color: [u8; 4],
}

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
        commands.push(HostPaintCommand::quad(
            expanded_offset_rect(rect, layer.offset_y, layer.grow),
            Some(clip.clone()),
            order + index as i32,
            Some(layer.color),
            None,
            0.0,
            (corner_radius + layer.grow).max(0.0),
            opacity,
        ));
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
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn paper_shadow_layers_project_from_host_shadow_color() {
        let mut palette = PALETTE;
        palette.shadow = [10, 20, 30, 200];

        let layers = shadow_layers_from_host(9.0, palette);

        assert_eq!(layers[0].color, [10, 20, 30, 54]);
        assert_eq!(layers[1].color, [10, 20, 30, 62]);
        assert_eq!(layers[2].color, [10, 20, 30, 88]);
    }

    #[test]
    fn paper_shadow_layers_keep_elevation_geometry() {
        let layers = shadow_layers_from_host(9.0, PALETTE);

        assert_eq!(layers[0].offset_y, 3.0);
        assert_eq!(layers[0].grow, 1.0);
        assert_eq!(layers[1].offset_y, 9.0);
        assert_eq!(layers[1].grow, 0.0);
        assert_eq!(layers[2].offset_y, 9.0);
        assert_eq!(layers[2].grow, 0.0);
    }
}
