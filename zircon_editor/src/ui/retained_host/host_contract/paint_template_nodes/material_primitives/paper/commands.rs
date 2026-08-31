use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::geometry::bounded_paper_rect;
use super::identity::is_paper_root_node;
use super::shadow::push_paper_shadow;
use super::style::{
    paper_background_color, paper_border_color, paper_border_width, paper_corner_radius,
    paper_dark_overlay, paper_elevation, paper_is_outlined,
};

const MAX_PAPER_COMMANDS: usize = 5;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_paper_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_paper_root_node(node) {
        return false;
    }

    let paper_rect = bounded_paper_rect(rect);
    if !paper_rect.x.is_finite()
        || !paper_rect.y.is_finite()
        || paper_rect.width <= 0.0
        || paper_rect.height <= 0.0
    {
        return true;
    }

    reserve_paper_command_capacity(commands);
    let outlined = paper_is_outlined(node);
    let elevation = paper_elevation(node);
    let corner_radius = paper_corner_radius(node, &paper_rect);

    if !outlined && elevation > 0.0 {
        push_paper_shadow(
            commands,
            &paper_rect,
            clip,
            order - 3,
            elevation,
            corner_radius,
            opacity,
        );
    }

    commands.push(HostPaintCommand::quad(
        paper_rect.clone(),
        Some(clip.clone()),
        order,
        Some(paper_background_color(node)),
        paper_border_color(node, outlined),
        paper_border_width(node, outlined),
        corner_radius,
        opacity,
    ));

    if !outlined && elevation > 0.0 && paper_background_color(node)[3] > 0 {
        commands.push(HostPaintCommand::quad(
            paper_rect,
            Some(clip.clone()),
            order + 1,
            Some(paper_dark_overlay(elevation)),
            None,
            0.0,
            corner_radius,
            opacity,
        ));
    }

    true
}

fn reserve_paper_command_capacity(commands: &mut Vec<HostPaintCommand>) {
    commands.reserve(MAX_PAPER_COMMANDS);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_finite_paper_origins_do_not_emit_surface_or_shadow_commands() {
        let node = TemplatePaneNodeData {
            component_role: "paper".to_owned(),
            elevation: 4.0,
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: 8.0,
            y: f32::NEG_INFINITY,
            width: 48.0,
            height: 24.0,
        };
        let mut commands = Vec::new();

        assert!(push_paper_primitive_commands(
            &mut commands,
            &node,
            &rect,
            &rect,
            0,
            1.0,
        ));
        assert!(commands.is_empty());
    }
}

#[cfg(test)]
#[path = "commands/reserve_capacity_tests.rs"]
mod reserve_capacity_tests;
