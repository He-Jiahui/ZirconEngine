use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_popup_row_adornments::{
    menu_row_adornment_kind, push_popup_row_adornment,
};
use super::super::content::popup_row_content_style;
use super::super::layers::{popup_row_adornment_order, popup_row_base_order};
use super::super::surface::{push_popup_background, push_popup_row_surface, push_popup_separator};
use super::super::text::{push_popup_row_label, push_popup_row_shortcut};
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
    push_popup_background(commands, rect, clip, order, opacity);

    for row in 0..row_count {
        let Some(row_rect) = menu_item_row_frame(rect, row_count, row) else {
            continue;
        };
        if intersect(&row_rect, clip).is_none() {
            continue;
        }
        let Some(item) = node.structured_menu_items.row_data(row) else {
            continue;
        };
        let row_order = popup_row_base_order(order, row);
        if item.separator {
            push_popup_separator(commands, &row_rect, clip, row_order, opacity);
            continue;
        }
        let style = popup_menu_row_style(&item);
        let content_style = popup_row_content_style(&style);
        let adornment = menu_row_adornment_kind(&item);
        push_popup_row_surface(commands, &row_rect, clip, row_order, style, opacity);
        push_popup_row_label(
            commands,
            &row_rect,
            clip,
            row_order,
            item.label.to_string(),
            content_style.text,
            adornment,
            opacity,
        );
        if !item.shortcut.is_empty() {
            push_popup_row_shortcut(
                commands,
                &row_rect,
                clip,
                row_order,
                item.shortcut.to_string(),
                content_style.shortcut,
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
