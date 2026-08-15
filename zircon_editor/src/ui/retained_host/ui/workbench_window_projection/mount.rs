use crate::ui::retained_host as host_contract;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::style::{ResolvedButtonStyle, StyleDimension};

pub(super) fn project_node_into_physical_mount(
    mut node: host_contract::TemplatePaneNodeData,
    mount_frame: Option<UiFrame>,
    scale_factor: f32,
) -> host_contract::TemplatePaneNodeData {
    let scale_factor = normalized_scale_factor(scale_factor);
    scale_node_metrics(&mut node, scale_factor);
    let mount_frame = mount_frame.unwrap_or_default();
    node.frame.x += mount_frame.x;
    node.frame.y += mount_frame.y;
    if node.has_clip_frame {
        node.clip_frame.x += mount_frame.x;
        node.clip_frame.y += mount_frame.y;
    }
    if node.has_popup_anchor {
        node.popup_anchor_x += mount_frame.x;
        node.popup_anchor_y += mount_frame.y;
    }
    node
}

fn scale_node_metrics(node: &mut host_contract::TemplatePaneNodeData, scale_factor: f32) {
    scale_frame(&mut node.frame, scale_factor);
    if node.has_clip_frame {
        scale_frame(&mut node.clip_frame, scale_factor);
    }
    if node.has_popup_anchor {
        node.popup_anchor_x = scale_visual_metric(node.popup_anchor_x, scale_factor);
        node.popup_anchor_y = scale_visual_metric(node.popup_anchor_y, scale_factor);
    }

    for value in [
        &mut node.layout_offset_x,
        &mut node.layout_offset_y,
        &mut node.layout_icon_size,
        &mut node.layout_content_offset_x,
        &mut node.layout_content_offset_y,
        &mut node.layout_first_cell_offset_x,
        &mut node.layout_fourth_cell_offset_x,
        &mut node.icon_stroke_width,
        &mut node.selected_segment_border_width,
        &mut node.selected_segment_underline_height,
        &mut node.tree_indent_px,
        &mut node.virtualization_item_extent,
        &mut node.ripple_pressed_x,
        &mut node.ripple_pressed_y,
        &mut node.drag_cursor_x,
        &mut node.drag_cursor_y,
        &mut node.drag_offset_x,
        &mut node.drag_offset_y,
        &mut node.drag_preview_width,
        &mut node.drag_preview_height,
        &mut node.drop_target_x,
        &mut node.drop_target_y,
        &mut node.drop_target_width,
        &mut node.drop_target_height,
        &mut node.font_size,
        &mut node.corner_radius,
        &mut node.border_width,
        &mut node.elevation,
    ] {
        *value = scale_visual_metric(*value, scale_factor);
    }
    scale_button_style(&mut node.button_style, scale_factor);
}

fn scale_frame(frame: &mut host_contract::TemplateNodeFrameData, scale_factor: f32) {
    frame.x = scale_visual_metric(frame.x, scale_factor);
    frame.y = scale_visual_metric(frame.y, scale_factor);
    frame.width = scale_visual_metric(frame.width, scale_factor);
    frame.height = scale_visual_metric(frame.height, scale_factor);
}

fn scale_button_style(style: &mut ResolvedButtonStyle, scale_factor: f32) {
    scale_dimension(&mut style.width, scale_factor);
    scale_dimension(&mut style.height, scale_factor);
    style.element.border_width = scale_visual_metric(style.element.border_width, scale_factor);
    style.element.corner_radius = scale_visual_metric(style.element.corner_radius, scale_factor);
    scale_dimension(&mut style.element.width, scale_factor);
    scale_dimension(&mut style.element.height, scale_factor);
}

fn scale_dimension(dimension: &mut StyleDimension, scale_factor: f32) {
    if let StyleDimension::Fixed(value) = dimension {
        *value = scale_visual_metric(*value, scale_factor);
    }
}

fn normalized_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

pub(super) fn scale_visual_metric(value: f32, scale_factor: f32) -> f32 {
    let scale_factor = normalized_scale_factor(scale_factor);
    let scaled = value * scale_factor;
    if scaled.is_finite() {
        scaled
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_projection_scales_geometry_and_visual_metrics_but_not_semantic_values() {
        let mut node = host_contract::TemplatePaneNodeData::default();
        node.frame = host_contract::TemplateNodeFrameData {
            x: 8.0,
            y: 12.0,
            width: 80.0,
            height: 40.0,
        };
        node.font_size = 12.0;
        node.corner_radius = 3.0;
        node.border_width = 1.0;
        node.value_number = 0.75;
        node.value_percent = 0.75;
        node.layout_second_cell_offset_x = 28.0;
        node.layout_third_cell_offset_x = 5.0;
        node.transition_duration_ms = 120;
        node.button_style.width = StyleDimension::Fixed(24.0);
        node.button_style.element.corner_radius = 2.0;

        let projected = project_node_into_physical_mount(
            node,
            Some(UiFrame::new(100.0, 50.0, 400.0, 200.0)),
            2.0,
        );

        assert_eq!(projected.frame.x, 116.0);
        assert_eq!(projected.frame.y, 74.0);
        assert_eq!(projected.frame.width, 160.0);
        assert_eq!(projected.font_size, 24.0);
        assert_eq!(projected.corner_radius, 6.0);
        assert_eq!(projected.border_width, 2.0);
        assert_eq!(projected.value_number, 0.75);
        assert_eq!(projected.value_percent, 0.75);
        assert_eq!(projected.layout_second_cell_offset_x, 28.0);
        assert_eq!(projected.layout_third_cell_offset_x, 5.0);
        assert_eq!(projected.transition_duration_ms, 120);
        assert_eq!(projected.button_style.width, StyleDimension::Fixed(48.0));
        assert_eq!(projected.button_style.element.corner_radius, 4.0);
    }
}
