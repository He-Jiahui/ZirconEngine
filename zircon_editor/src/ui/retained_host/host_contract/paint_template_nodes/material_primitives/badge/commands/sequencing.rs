use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::identity::{is_badge_root_node, is_badge_slot_node};
use super::overlay::push_badge_overlay;
use super::root_label::push_badge_root_label;
use super::root_surface::push_badge_root_surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_badge_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_badge_slot_node(node) {
        return true;
    }
    if !is_badge_root_node(node) {
        return false;
    }
    if !rect.x.is_finite()
        || !rect.y.is_finite()
        || !rect.width.is_finite()
        || !rect.height.is_finite()
        || rect.width <= 0.0
        || rect.height <= 0.0
    {
        return true;
    }

    push_badge_root_surface(commands, node, rect, clip, order, opacity);
    push_badge_root_label(commands, node, rect, clip, order + 1, opacity);
    push_badge_overlay(commands, node, rect, clip, order + 2, opacity);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_finite_badge_root_layout_does_not_emit_any_paint_commands() {
        let node = TemplatePaneNodeData {
            component_role: "badge".to_owned(),
            text: "Asset status".to_owned(),
            value_text: "1".to_owned(),
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: 8.0,
            y: f32::NAN,
            width: 48.0,
            height: 24.0,
        };
        let mut commands = Vec::new();

        assert!(push_badge_primitive_commands(
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
