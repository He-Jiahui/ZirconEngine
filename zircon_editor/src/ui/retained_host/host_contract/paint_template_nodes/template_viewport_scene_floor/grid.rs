use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_style::surface_color;

const GRID_GLOW: [u8; 4] = [134, 161, 167, 28];
const GRID_MAJOR_GLOW: [u8; 4] = [162, 188, 192, 42];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_floor_grid_line(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let major = is_major_grid_line(node.control_id.as_str());
    let glow_color = if major { GRID_MAJOR_GLOW } else { GRID_GLOW };
    let glow_rect = if rect.width >= rect.height {
        FrameRect {
            x: rect.x,
            y: rect.y - 1.0,
            width: rect.width,
            height: rect.height + 2.0,
        }
    } else {
        FrameRect {
            x: rect.x - 1.0,
            y: rect.y,
            width: rect.width + 2.0,
            height: rect.height,
        }
    };
    commands.push(HostPaintCommand::quad(
        glow_rect,
        Some(clip.clone()),
        order,
        Some(glow_color),
        None,
        0.0,
        0.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order + 1,
        Some(surface_color(node)),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn is_major_grid_line(control_id: &str) -> bool {
    let bytes = control_id.as_bytes();
    let Some(suffix) = bytes.get(bytes.len().saturating_sub(2)..) else {
        return false;
    };
    matches!(
        suffix,
        [b'H', b'2'] | [b'H', b'4'] | [b'V', b'2'] | [b'V', b'5']
    )
}

#[cfg(test)]
#[path = "tests/grid_optimization_batch_hd_editor585_tests.rs"]
mod optimization_batch_hd_editor585_tests;
