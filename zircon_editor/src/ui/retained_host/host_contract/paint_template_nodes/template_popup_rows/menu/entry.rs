use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_popup_row_adornments::{
    menu_row_adornment_kind, push_popup_row_adornment,
};
use super::super::surface::{
    push_popup_background, push_popup_row_surface, push_popup_separator, POPUP_ROW_ORDER_OFFSET,
};
use super::super::text::{push_popup_row_label, push_popup_row_shortcut};
use super::style::popup_menu_row_style;
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
        let Some(item) = node.structured_menu_items.row_data(row) else {
            continue;
        };
        let Some(row_rect) = menu_item_row_frame(rect, row_count, row) else {
            continue;
        };
        if item.separator {
            push_popup_separator(commands, &row_rect, clip, order + row as i32, opacity);
            continue;
        }
        let style = popup_menu_row_style(&item);
        push_popup_row_surface(
            commands,
            &row_rect,
            clip,
            order + row as i32,
            style,
            opacity,
        );
        push_popup_row_label(
            commands,
            &row_rect,
            clip,
            order + row as i32,
            item.label.to_string(),
            style.text,
            menu_row_adornment_kind(&item),
            opacity,
        );
        if !item.shortcut.is_empty() {
            push_popup_row_shortcut(
                commands,
                &row_rect,
                clip,
                order + row as i32,
                item.shortcut.to_string(),
                style.shortcut,
                opacity,
            );
        }
        if let Some(adornment) = menu_row_adornment_kind(&item) {
            push_popup_row_adornment(
                commands,
                &row_rect,
                clip,
                order + row as i32 + POPUP_ROW_ORDER_OFFSET + 4,
                adornment,
                style.adornment,
                opacity,
            );
        }
    }
}
