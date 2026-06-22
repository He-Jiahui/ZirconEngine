use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_alerts::push_alert_commands;
use super::super::super::template_axis_labels::push_axis_label_commands;
use super::super::super::template_axis_value_fields::push_axis_value_field_commands;
use super::super::super::template_chips::push_chip_commands;
use super::super::super::template_command_palette::push_command_palette_commands;
use super::super::super::template_dialogs::push_dialog_commands;
use super::super::super::template_drag_overlay::push_drag_overlay_commands;
use super::super::super::template_fields::push_field_commands;
use super::super::super::template_icon_buttons::push_icon_button_commands;
use super::super::super::template_inspector_rows::push_inspector_row_commands;
use super::super::super::template_list_rows::push_list_row_commands;
use super::super::super::template_notification_center::push_notification_center_commands;
use super::super::super::template_section_titles::push_section_title_commands;
use super::super::super::template_sliders::push_slider_commands;
use super::super::super::template_status_controls::push_status_control_commands;
use super::super::super::template_table_rows::push_table_row_commands;
use super::super::super::template_tooltips::push_tooltip_commands;
use super::super::super::template_tree_rows::push_tree_row_commands;
use super::super::super::template_viewport_scene::push_viewport_scene_commands;

pub(super) fn push_secondary_specialized_template_node_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    node_clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    push_tree_row_commands(commands, node, rect, node_clip, order, opacity)
        || push_list_row_commands(commands, node, rect, node_clip, order, opacity)
        || push_table_row_commands(commands, node, rect, node_clip, order, opacity)
        || push_status_control_commands(commands, node, rect, node_clip, order, opacity)
        || push_chip_commands(commands, node, rect, node_clip, order, opacity)
        || push_viewport_scene_commands(commands, node, rect, node_clip, order, opacity)
        || push_section_title_commands(commands, node, rect, node_clip, order, opacity)
        || push_icon_button_commands(commands, node, rect, node_clip, order, opacity)
        || push_inspector_row_commands(commands, node, rect, node_clip, order, opacity)
        || push_axis_value_field_commands(commands, node, rect, node_clip, order, opacity)
        || push_axis_label_commands(commands, node, rect, node_clip, order, opacity)
        || push_field_commands(commands, node, rect, node_clip, order, opacity)
        || push_slider_commands(commands, node, rect, node_clip, order, opacity)
        || push_alert_commands(commands, node, rect, node_clip, order, opacity)
        || push_dialog_commands(commands, node, rect, node_clip, order, opacity)
        || push_command_palette_commands(commands, node, rect, node_clip, order, opacity)
        || push_notification_center_commands(commands, node, rect, node_clip, order, opacity)
        || push_drag_overlay_commands(commands, node, rect, node_clip, order, opacity)
        || push_tooltip_commands(commands, node, rect, node_clip, order, opacity)
}
