use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_popup_row_adornments::{
    option_adornment_kind, push_popup_row_adornment, PopupRowAdornmentKind,
};
use super::super::surface::{
    push_popup_background, push_popup_row_surface, POPUP_ROW_ORDER_OFFSET,
};
use super::super::text::push_popup_row_label;
use super::style::{popup_option_row_marked, popup_option_row_style};
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
    push_popup_background(commands, &popup_rect, clip, order, opacity);

    for row in 0..row_count {
        let Some(option) = node.structured_options.row_data(row) else {
            continue;
        };
        let Some(row_rect) = template_option_row_frame_within(node, rect, row_count, row, bounds)
        else {
            continue;
        };
        let style = popup_option_row_style(&option);
        let selected = popup_option_row_marked(&option);
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
            option.label.to_string(),
            style.text,
            option_adornment_kind(selected),
            opacity,
        );
        if selected {
            push_popup_row_adornment(
                commands,
                &row_rect,
                clip,
                order + row as i32 + POPUP_ROW_ORDER_OFFSET + 4,
                PopupRowAdornmentKind::Check,
                style.adornment,
                opacity,
            );
        }
    }
}
