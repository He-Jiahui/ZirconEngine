use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::geometry::divider_is_vertical;
use super::horizontal::push_horizontal_divider;
use super::identity::is_divider_node;
use super::vertical::push_vertical_divider;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_divider_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_divider_node(node) {
        return false;
    }
    let Some(clip) = intersect(rect, clip) else {
        return true;
    };

    // MUI Divider is border/pseudo-element geometry, not a filled panel.
    // Emit explicit line segments so inset, middle, and label gaps match the web contract.
    if divider_is_vertical(node, rect) {
        push_vertical_divider(commands, node, rect, &clip, order, opacity);
    } else {
        push_horizontal_divider(commands, node, rect, &clip, order, opacity);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn divider_node() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            role: "Divider".into(),
            ..TemplatePaneNodeData::default()
        }
    }

    #[test]
    fn fully_clipped_divider_does_not_emit_paint_commands() {
        let rect = frame(8.0, 6.0, 160.0, 24.0);
        let clip = frame(200.0, 0.0, 80.0, 80.0);
        let mut commands = Vec::new();

        assert!(push_divider_primitive_commands(
            &mut commands,
            &divider_node(),
            &rect,
            &clip,
            2,
            1.0,
        ));

        assert!(commands.is_empty());
    }

    #[test]
    fn partially_clipped_divider_keeps_only_clipped_paint_commands() {
        let rect = frame(8.0, 6.0, 160.0, 24.0);
        let clip = frame(16.0, 8.0, 60.0, 18.0);
        let mut commands = Vec::new();

        assert!(push_divider_primitive_commands(
            &mut commands,
            &divider_node(),
            &rect,
            &clip,
            2,
            1.0,
        ));

        assert!(!commands.is_empty());
        assert!(commands.iter().all(|command| command
            .clip_frame
            .as_ref()
            .is_some_and(|clip_frame| frame_is_within(&clip, clip_frame))));
    }

    fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
        FrameRect {
            x,
            y,
            width,
            height,
        }
    }

    fn frame_is_within(outer: &FrameRect, inner: &FrameRect) -> bool {
        inner.x >= outer.x
            && inner.y >= outer.y
            && inner.x + inner.width <= outer.x + outer.width
            && inner.y + inner.height <= outer.y + outer.height
    }
}
