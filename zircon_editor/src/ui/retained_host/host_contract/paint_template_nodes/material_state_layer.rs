use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;

mod ripple;
mod state;

use ripple::{push_ripple_commands, ripple_is_visible};
use state::{state_layer_color, state_layer_opacity};

#[cfg(test)]
use ripple::{ripple_diameter, ripple_rect, RIPPLE_DIAMETER_EXPANSION};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_state_layer_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    corner_radius: f32,
    order: i32,
    opacity_multiplier: f32,
) {
    let overlay_opacity = state_layer_opacity(node);
    let ripple_visible = ripple_is_visible(node);
    if overlay_opacity.is_none() && !ripple_visible {
        return;
    }

    let color = state_layer_color(node);
    if let Some(opacity) = overlay_opacity {
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            corner_radius,
            opacity * opacity_multiplier,
        ));
    }

    if ripple_visible {
        push_ripple_commands(
            commands,
            node,
            rect,
            clip,
            order + 1,
            color,
            opacity_multiplier,
        );
    }
}

#[cfg(test)]
#[path = "material_state_layer_tests/mod.rs"]
mod tests;
