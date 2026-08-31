use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_popup_row_adornments::{
    option_adornment_kind, push_popup_row_adornment,
};
use super::super::content::popup_row_content_style;
use super::super::layers::{popup_row_adornment_order, popup_row_base_order};
use super::super::metrics::workbench_popup_row_metrics;
use super::super::surface::{push_popup_background, push_popup_row_surface};
use super::super::text::{popup_row_text_columns, push_popup_row_label};
use super::style::{popup_option_row_marked, popup_option_row_style};
use crate::ui::retained_host::host_contract::paint_geometry::intersect;
use crate::ui::retained_host::host_contract::template_popup_layout::{
    template_option_popup_frame_within, template_option_row_frame_within,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_option_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    bounds: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let row_count = node.structured_options.row_count();
    if row_count == 0 {
        return;
    }
    let Some(popup_rect) = template_option_popup_frame_within(node, rect, row_count, bounds) else {
        return;
    };
    push_popup_background(commands, node, &popup_rect, clip, order, opacity);
    let row_metrics = workbench_popup_row_metrics();

    for row in 0..row_count {
        let Some(row_rect) = template_option_row_frame_within(node, rect, row_count, row, bounds)
        else {
            continue;
        };
        if intersect(&row_rect, clip).is_none() {
            continue;
        }
        let Some(option) = node.structured_options.get(row) else {
            continue;
        };
        let row_order = popup_row_base_order(order, row);
        let style = popup_option_row_style(option);
        let content_style = popup_row_content_style(&style);
        let selected = popup_option_row_marked(option);
        let adornment = option_adornment_kind(selected);
        let text_columns = popup_row_text_columns(&row_rect, &row_metrics, "", adornment.is_some());
        push_popup_row_surface(commands, &row_rect, clip, row_order, style, opacity);
        push_popup_row_label(
            commands,
            &row_rect,
            &text_columns.label,
            clip,
            row_order,
            option.label.to_string(),
            content_style.text,
            &row_metrics,
            opacity,
        );
        if let Some(adornment) = adornment {
            push_popup_row_adornment(
                commands,
                &row_rect,
                clip,
                popup_row_adornment_order(row_order),
                adornment,
                content_style.adornment,
                opacity,
            );
        }
    }
}
