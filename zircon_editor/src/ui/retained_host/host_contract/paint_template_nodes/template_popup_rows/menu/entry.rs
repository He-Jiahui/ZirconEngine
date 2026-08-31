use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_popup_row_adornments::{
    menu_row_adornment_kind, push_popup_row_adornment,
};
use super::super::content::popup_row_content_style;
use super::super::layers::{popup_row_adornment_order, popup_row_base_order};
use super::super::metrics::workbench_popup_row_metrics;
use super::super::surface::{push_popup_background, push_popup_row_surface, push_popup_separator};
use super::super::text::{popup_row_text_columns, push_popup_row_label, push_popup_row_shortcut};
use super::style::popup_menu_row_style;
use crate::ui::retained_host::host_contract::paint_geometry::intersect;
use crate::ui::retained_host::host_contract::template_popup_layout::menu_item_row_frame;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_menu_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let row_count = node.structured_menu_items.row_count();
    if row_count == 0 {
        return;
    }
    push_popup_background(commands, node, rect, clip, order, opacity);
    let row_metrics = workbench_popup_row_metrics();

    for row in 0..row_count {
        let Some(row_rect) = menu_item_row_frame(node, rect, row_count, row) else {
            continue;
        };
        if intersect(&row_rect, clip).is_none() {
            continue;
        }
        let Some(item) = node.structured_menu_items.get(row) else {
            continue;
        };
        let row_order = popup_row_base_order(order, row);
        if item.separator {
            push_popup_separator(commands, &row_rect, clip, row_order, opacity);
            continue;
        }
        let style = popup_menu_row_style(item);
        let content_style = popup_row_content_style(&style);
        let adornment = menu_row_adornment_kind(item);
        let text_columns = popup_row_text_columns(
            &row_rect,
            &row_metrics,
            item.shortcut.as_str(),
            adornment.is_some(),
        );
        push_popup_row_surface(commands, &row_rect, clip, row_order, style, opacity);
        push_popup_row_label(
            commands,
            &row_rect,
            &text_columns.label,
            clip,
            row_order,
            item.label.to_string(),
            content_style.text,
            &row_metrics,
            opacity,
        );
        if let Some(shortcut_rect) = text_columns.shortcut.as_ref() {
            push_popup_row_shortcut(
                commands,
                &row_rect,
                shortcut_rect,
                clip,
                row_order,
                item.shortcut.to_string(),
                content_style.shortcut,
                &row_metrics,
                opacity,
            );
        }
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
