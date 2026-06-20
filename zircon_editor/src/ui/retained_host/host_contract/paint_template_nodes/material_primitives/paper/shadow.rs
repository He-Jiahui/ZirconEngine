use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;

const MUI_SHADOW_UMBRA: [u8; 4] = [0, 0, 0, 51];
const MUI_SHADOW_PENUMBRA: [u8; 4] = [0, 0, 0, 36];
const MUI_SHADOW_AMBIENT: [u8; 4] = [0, 0, 0, 31];

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
    let elevation = elevation.clamp(1.0, 24.0);
    let offset = elevation.round().max(1.0);
    [
        ShadowLayer {
            offset_y: (elevation / 3.0).round().max(1.0),
            grow: 1.0,
            color: MUI_SHADOW_AMBIENT,
        },
        ShadowLayer {
            offset_y: offset,
            grow: 0.0,
            color: MUI_SHADOW_PENUMBRA,
        },
        ShadowLayer {
            offset_y: offset,
            grow: 0.0,
            color: MUI_SHADOW_UMBRA,
        },
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
