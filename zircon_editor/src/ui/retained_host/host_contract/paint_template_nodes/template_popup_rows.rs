use super::super::data::{
    FrameRect, TemplatePaneMenuItemData, TemplatePaneNodeData, TemplatePaneOptionData,
};
use super::super::paint_geometry::intersect;
use super::super::paint_theme::PALETTE;
use super::super::template_popup_layout::{
    menu_item_row_frame, template_option_popup_frame_within, template_option_row_frame_within,
};
use super::render_commands::HostPaintCommand;
use super::style_selector::{
    select_workbench_popup_row_style, WorkbenchPopupRowState, WorkbenchPopupRowStyle,
};
use super::template_popup_row_adornments::{
    menu_item_has_flag, menu_row_adornment_kind, option_adornment_kind, push_popup_row_adornment,
    PopupRowAdornmentKind, POPUP_ROW_ADORNMENT_RESERVED_WIDTH,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const DEFAULT_POPUP_FONT_SIZE: f32 = 12.0;
const MIN_TEXT_RECT_HEIGHT: f32 = 12.0;
const POPUP_ROW_TEXT_X: f32 = 9.0;
const POPUP_ROW_TEXT_Y: f32 = 5.0;
const POPUP_ROW_SELECTED_MARK_WIDTH: f32 = 3.0;
const POPUP_ROW_ORDER_OFFSET: i32 = 10_000;

pub(super) fn push_template_popup_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    bounds: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !node.popup_open {
        return;
    }
    if node.structured_menu_items.row_count() > 0 {
        push_menu_row_commands(commands, node, rect, clip, order, opacity);
    } else if node.structured_options.row_count() > 0 {
        push_option_row_commands(commands, node, rect, bounds, clip, order, opacity);
    }
}

fn push_option_row_commands(
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

fn push_menu_row_commands(
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

fn push_popup_background(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if intersect(rect, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order + POPUP_ROW_ORDER_OFFSET,
        Some(PALETTE.popup),
        Some(PALETTE.border),
        1.0,
        5.0,
        opacity,
    ));
}

fn push_popup_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    style: WorkbenchPopupRowStyle,
    opacity: f32,
) {
    if intersect(row_rect, clip).is_none() {
        return;
    }
    if let Some(background) = style.background {
        commands.push(HostPaintCommand::quad(
            row_rect.clone(),
            Some(clip.clone()),
            order + POPUP_ROW_ORDER_OFFSET + 1,
            Some(background),
            None,
            0.0,
            3.0,
            opacity,
        ));
    }
    if let Some(selection_mark) = style.selection_mark {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: row_rect.x,
                y: row_rect.y + 4.0,
                width: POPUP_ROW_SELECTED_MARK_WIDTH,
                height: (row_rect.height - 8.0).max(1.0),
            },
            Some(clip.clone()),
            order + POPUP_ROW_ORDER_OFFSET + 2,
            Some(selection_mark),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn push_popup_separator(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let separator = FrameRect {
        x: row_rect.x + 8.0,
        y: row_rect.y + row_rect.height * 0.5,
        width: (row_rect.width - 16.0).max(1.0),
        height: 1.0,
    };
    if intersect(&separator, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::quad(
        separator,
        Some(clip.clone()),
        order + POPUP_ROW_ORDER_OFFSET + 2,
        Some(PALETTE.border),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn push_popup_row_label(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: String,
    color: [u8; 4],
    adornment: Option<PopupRowAdornmentKind>,
    opacity: f32,
) {
    if label.is_empty() || intersect(row_rect, clip).is_none() {
        return;
    }
    let right_reserved = if adornment.is_some() {
        POPUP_ROW_ADORNMENT_RESERVED_WIDTH
    } else {
        POPUP_ROW_TEXT_X
    };
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: row_rect.x + POPUP_ROW_TEXT_X,
            y: row_rect.y + POPUP_ROW_TEXT_Y,
            width: (row_rect.width - POPUP_ROW_TEXT_X - right_reserved).max(1.0),
            height: (row_rect.height - POPUP_ROW_TEXT_Y * 2.0).max(MIN_TEXT_RECT_HEIGHT),
        },
        Some(clip.clone()),
        order + POPUP_ROW_ORDER_OFFSET + 3,
        label,
        color,
        DEFAULT_POPUP_FONT_SIZE,
        DEFAULT_POPUP_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_popup_row_shortcut(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    shortcut: String,
    color: [u8; 4],
    opacity: f32,
) {
    if shortcut.is_empty() || intersect(row_rect, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: row_rect.x + row_rect.width * 0.58,
            y: row_rect.y + POPUP_ROW_TEXT_Y,
            width: (row_rect.width * 0.38).max(1.0),
            height: (row_rect.height - POPUP_ROW_TEXT_Y * 2.0).max(MIN_TEXT_RECT_HEIGHT),
        },
        Some(clip.clone()),
        order + POPUP_ROW_ORDER_OFFSET + 3,
        shortcut,
        color,
        DEFAULT_POPUP_FONT_SIZE,
        DEFAULT_POPUP_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn popup_option_row_marked(option: &TemplatePaneOptionData) -> bool {
    option.selected || option.special
}

fn popup_option_row_style(option: &TemplatePaneOptionData) -> WorkbenchPopupRowStyle {
    select_workbench_popup_row_style(WorkbenchPopupRowState {
        hovered: option.hovered,
        pressed: option.pressed,
        focused: option.focused,
        disabled: option.disabled,
        loading: option.loading,
        selected: popup_option_row_marked(option),
        ..WorkbenchPopupRowState::default()
    })
}

fn popup_menu_row_style(item: &TemplatePaneMenuItemData) -> WorkbenchPopupRowStyle {
    select_workbench_popup_row_style(WorkbenchPopupRowState {
        hovered: item.hovered,
        pressed: item.pressed,
        focused: item.focused,
        disabled: item.disabled,
        checked: item.checked,
        loading: item.loading || menu_item_has_flag(item, "loading"),
        danger: menu_item_has_flag(item, "danger"),
        ..WorkbenchPopupRowState::default()
    })
}

#[cfg(test)]
#[path = "template_popup_rows_tests.rs"]
mod tests;
