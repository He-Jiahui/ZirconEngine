#[cfg(test)]
use crate::ui::retained_host::primitives::ModelRc;

use super::super::data::{FrameRect, HostTextInputFocusData, TemplatePaneNodeData};
use super::super::paint_geometry::{frame_from_template, intersect, is_visible_frame, translated};
use super::material_primitives::{
    push_material_primitive_commands, push_material_text_field_surface_commands,
};
use super::mui_x_primitives::push_mui_x_primitive_commands;
use super::render_commands::HostPaintCommand;
use super::template_alerts::push_alert_commands;
use super::template_axis_labels::push_axis_label_commands;
use super::template_axis_value_fields::push_axis_value_field_commands;
use super::template_buttons::push_button_commands;
use super::template_chips::push_chip_commands;
use super::template_command_palette::push_command_palette_commands;
use super::template_dialogs::push_dialog_commands;
use super::template_drag_overlay::push_drag_overlay_commands;
use super::template_dropdowns::{dropdown_paint_rect, push_dropdown_commands};
use super::template_fields::push_field_commands;
use super::template_icon_buttons::push_icon_button_commands;
use super::template_inspector_rows::push_inspector_row_commands;
use super::template_list_rows::push_list_row_commands;
use super::template_material_feedback::push_material_feedback_primitive_commands;
use super::template_node_images::push_template_image_command;
use super::template_node_surface::{is_frame_only_node, push_template_surface_fallback_commands};
use super::template_node_text::push_template_text_fallback_command;
use super::template_notification_center::push_notification_center_commands;
use super::template_popup_rows::push_template_popup_row_commands;
use super::template_property_rows::push_property_row_text_commands;
use super::template_section_titles::push_section_title_commands;
use super::template_segmented_controls::push_segmented_control_commands;
use super::template_selection_controls::push_selection_control_commands;
use super::template_shell_panels::push_shell_panel_commands;
use super::template_sliders::push_slider_commands;
use super::template_status_controls::push_status_control_commands;
use super::template_table_rows::{push_table_row_commands, push_table_row_text_commands};
use super::template_tooltips::push_tooltip_commands;
use super::template_tree_rows::push_tree_row_commands;
use super::template_viewport_scene::push_viewport_scene_commands;

const TEMPLATE_NODE_ORDER_STRIDE: i32 = 4;
const TEMPLATE_NODE_Z_LAYER_STRIDE: i32 = 100_000;

#[cfg(test)]
pub(crate) fn paint_template_nodes_for_test(
    width: u32,
    height: u32,
    nodes: ModelRc<TemplatePaneNodeData>,
) -> Vec<u8> {
    super::template_node_pipeline::paint_template_nodes_for_test(width, height, nodes)
}

#[cfg(test)]
pub(crate) fn paint_template_nodes_for_test_with_background(
    width: u32,
    height: u32,
    background: [u8; 4],
    nodes: ModelRc<TemplatePaneNodeData>,
) -> Vec<u8> {
    super::template_node_pipeline::paint_template_nodes_for_test_with_background(
        width, height, background, nodes,
    )
}

pub(super) fn push_template_node_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
    order: i32,
) {
    let local = frame_from_template(&node.frame);
    let rect = translated(&local, origin.x, origin.y);
    if !is_visible_frame(&rect) {
        return;
    }
    let Some(node_clip) = template_node_clip(node, origin, clip) else {
        return;
    };
    if intersect(&rect, &node_clip).is_none() {
        return;
    }
    if is_frame_only_node(node) {
        return;
    }

    let order = template_node_paint_order(node, order);
    let opacity = template_node_transition_opacity(node);
    if opacity <= 0.0 {
        return;
    }

    if push_material_feedback_primitive_commands(commands, node, &rect, &node_clip, order, opacity)
    {
        return;
    }

    if push_shell_panel_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_selection_control_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_segmented_control_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_button_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_dropdown_commands(commands, node, &rect, &node_clip, order, opacity) {
        let popup_anchor = dropdown_paint_rect(node, &rect);
        push_template_popup_row_commands(
            commands,
            node,
            &popup_anchor,
            origin,
            clip,
            order,
            opacity,
        );
        return;
    }

    if push_tree_row_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_list_row_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_table_row_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_status_control_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_chip_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_viewport_scene_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_section_title_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_icon_button_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_inspector_row_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_axis_value_field_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_axis_label_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_field_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_slider_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_alert_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_dialog_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_command_palette_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_notification_center_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_drag_overlay_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_tooltip_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    if push_material_primitive_commands(commands, node, &rect, &node_clip, order, opacity) {
        return;
    }

    let draws_mui_x_primitive =
        push_mui_x_primitive_commands(commands, node, &rect, &node_clip, order, opacity);

    let draws_text_field_surface = push_material_text_field_surface_commands(
        commands, node, &rect, &node_clip, order, opacity,
    );

    push_template_surface_fallback_commands(
        commands,
        node,
        &rect,
        &node_clip,
        order,
        opacity,
        draws_mui_x_primitive || draws_text_field_surface,
    );

    push_template_image_command(commands, node, &rect, &node_clip, order + 2, opacity);

    let property_row_text_painted =
        push_property_row_text_commands(commands, node, &rect, &node_clip, order + 3, opacity);
    let table_row_text_painted = !property_row_text_painted
        && push_table_row_text_commands(commands, node, &rect, &node_clip, order + 3, opacity);

    push_template_text_fallback_command(
        commands,
        node,
        &rect,
        &node_clip,
        order + 3,
        text_input_focus,
        property_row_text_painted,
        table_row_text_painted,
        opacity,
    );

    push_template_popup_row_commands(commands, node, &rect, origin, clip, order, opacity);
}

fn template_node_paint_order(node: &TemplatePaneNodeData, row_order: i32) -> i32 {
    node.z_index
        .saturating_mul(TEMPLATE_NODE_Z_LAYER_STRIDE)
        .saturating_add(row_order.saturating_mul(TEMPLATE_NODE_ORDER_STRIDE))
}

fn template_node_transition_opacity(node: &TemplatePaneNodeData) -> f32 {
    match node.transition_kind.as_str() {
        "fade" | "grow" | "zoom" => node.transition_progress.clamp(0.0, 1.0),
        _ => 1.0,
    }
}

fn template_node_clip(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    pane_clip: &FrameRect,
) -> Option<FrameRect> {
    let node_clip = if node.has_clip_frame {
        translated(&frame_from_template(&node.clip_frame), origin.x, origin.y)
    } else {
        pane_clip.clone()
    };
    intersect(&node_clip, pane_clip)
}
