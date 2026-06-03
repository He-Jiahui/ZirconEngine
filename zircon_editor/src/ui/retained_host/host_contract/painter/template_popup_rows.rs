use super::super::data::{
    FrameRect, TemplatePaneMenuItemData, TemplatePaneNodeData, TemplatePaneOptionData,
};
use super::super::template_popup_layout::{
    dropdown_option_popup_frame_within, dropdown_option_row_frame_within, menu_item_row_frame,
};
use super::geometry::intersect;
use super::render_commands::HostPaintCommand;
#[cfg(test)]
use super::style_selector::WORKBENCH_POPUP_ROW_DANGER_TEXT as POPUP_ROW_DANGER_TEXT;
use super::style_selector::{
    select_workbench_popup_row_style, WorkbenchPopupRowState, WorkbenchPopupRowStyle,
};
use super::theme::PALETTE;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const DEFAULT_POPUP_FONT_SIZE: f32 = 12.0;
const MIN_TEXT_RECT_HEIGHT: f32 = 12.0;
const POPUP_ROW_TEXT_X: f32 = 9.0;
const POPUP_ROW_TEXT_Y: f32 = 5.0;
const POPUP_ROW_SELECTED_MARK_WIDTH: f32 = 3.0;
const POPUP_ROW_ORDER_OFFSET: i32 = 10_000;
const POPUP_ROW_ADORNMENT_RIGHT: f32 = 12.0;
const POPUP_ROW_ADORNMENT_SIZE: f32 = 14.0;
const POPUP_ROW_ADORNMENT_RESERVED_WIDTH: f32 = 30.0;

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
    let Some(popup_rect) = dropdown_option_popup_frame_within(rect, row_count, bounds) else {
        return;
    };
    push_popup_background(commands, &popup_rect, clip, order, opacity);

    for row in 0..row_count {
        let Some(option) = node.structured_options.row_data(row) else {
            continue;
        };
        let Some(row_rect) = dropdown_option_row_frame_within(rect, row_count, row, bounds) else {
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
                order + row as i32,
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
                order + row as i32,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PopupRowAdornmentKind {
    Check,
    Chevron,
    Plus,
    Folder,
    Save,
    Trash,
}

fn option_adornment_kind(selected: bool) -> Option<PopupRowAdornmentKind> {
    selected.then_some(PopupRowAdornmentKind::Check)
}

fn menu_row_adornment_kind(
    item: &super::super::data::TemplatePaneMenuItemData,
) -> Option<PopupRowAdornmentKind> {
    if menu_item_has_flag(item, "submenu") {
        return Some(PopupRowAdornmentKind::Chevron);
    }
    if item.checked {
        return Some(PopupRowAdornmentKind::Check);
    }
    if let Some(icon) = menu_item_flag_value(item, "icon") {
        return popup_row_adornment_from_icon(&icon);
    }
    menu_item_default_icon(item.label.as_str()).and_then(popup_row_adornment_from_icon)
}

fn popup_row_adornment_from_icon(icon: &str) -> Option<PopupRowAdornmentKind> {
    match icon.trim().to_ascii_lowercase().as_str() {
        "add" | "new" | "plus" => Some(PopupRowAdornmentKind::Plus),
        "open" | "folder" => Some(PopupRowAdornmentKind::Folder),
        "save" | "disk" => Some(PopupRowAdornmentKind::Save),
        "delete" | "remove" | "trash" => Some(PopupRowAdornmentKind::Trash),
        "submenu" | "more" | "chevron" => Some(PopupRowAdornmentKind::Chevron),
        "check" | "checked" => Some(PopupRowAdornmentKind::Check),
        _ => None,
    }
}

fn menu_item_default_icon(label: &str) -> Option<&'static str> {
    match label.trim().to_ascii_lowercase().as_str() {
        "new" => Some("plus"),
        "open" => Some("folder"),
        "save" => Some("save"),
        "delete" => Some("trash"),
        "more tools" => Some("submenu"),
        _ => None,
    }
}

fn push_popup_row_adornment(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: PopupRowAdornmentKind,
    color: [u8; 4],
    opacity: f32,
) {
    if intersect(row_rect, clip).is_none() {
        return;
    }
    let rect = popup_row_adornment_rect(row_rect);
    let order = order + POPUP_ROW_ORDER_OFFSET + 4;
    match kind {
        PopupRowAdornmentKind::Check => {
            push_check_adornment(commands, &rect, clip, order, color, opacity);
        }
        PopupRowAdornmentKind::Chevron => {
            push_chevron_adornment(commands, &rect, clip, order, color, opacity);
        }
        PopupRowAdornmentKind::Plus => {
            push_plus_adornment(commands, &rect, clip, order, color, opacity);
        }
        PopupRowAdornmentKind::Folder => {
            push_folder_adornment(commands, &rect, clip, order, color, opacity);
        }
        PopupRowAdornmentKind::Save => {
            push_save_adornment(commands, &rect, clip, order, color, opacity);
        }
        PopupRowAdornmentKind::Trash => {
            push_trash_adornment(commands, &rect, clip, order, color, opacity);
        }
    }
}

fn popup_row_adornment_rect(row_rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: row_rect.x + row_rect.width - POPUP_ROW_ADORNMENT_RIGHT - POPUP_ROW_ADORNMENT_SIZE,
        y: row_rect.y + (row_rect.height - POPUP_ROW_ADORNMENT_SIZE).max(0.0) * 0.5,
        width: POPUP_ROW_ADORNMENT_SIZE,
        height: POPUP_ROW_ADORNMENT_SIZE,
    }
}

fn push_check_adornment(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 2.0, 7.0, 3.0, 2.0),
            local_rect(rect, 4.0, 9.0, 3.0, 2.0),
            local_rect(rect, 7.0, 4.0, 3.0, 7.0),
        ],
    );
}

fn push_chevron_adornment(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 5.0, 3.0, 2.0, 3.0),
            local_rect(rect, 7.0, 6.0, 2.0, 2.0),
            local_rect(rect, 5.0, 8.0, 2.0, 3.0),
        ],
    );
}

fn push_plus_adornment(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 6.0, 3.0, 2.0, 8.0),
            local_rect(rect, 3.0, 6.0, 8.0, 2.0),
        ],
    );
}

fn push_folder_adornment(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        local_rect(rect, 2.0, 5.0, 10.0, 7.0),
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        1.5,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        local_rect(rect, 3.0, 3.0, 5.0, 3.0),
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        1.0,
        opacity,
    ));
}

fn push_save_adornment(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        local_rect(rect, 2.0, 2.0, 10.0, 10.0),
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        1.5,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        local_rect(rect, 4.0, 3.0, 5.0, 3.0),
        Some(clip.clone()),
        order + 1,
        Some(PALETTE.popup),
        None,
        0.0,
        0.5,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        local_rect(rect, 4.0, 9.0, 6.0, 2.0),
        Some(clip.clone()),
        order + 1,
        Some(PALETTE.popup),
        None,
        0.0,
        0.5,
        opacity,
    ));
}

fn push_trash_adornment(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 3.0, 4.0, 8.0, 2.0),
            local_rect(rect, 5.0, 2.0, 4.0, 2.0),
            local_rect(rect, 4.0, 6.0, 6.0, 7.0),
            local_rect(rect, 6.0, 7.0, 1.0, 5.0),
            local_rect(rect, 8.0, 7.0, 1.0, 5.0),
        ],
    );
}

fn push_segments(
    commands: &mut Vec<HostPaintCommand>,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[FrameRect],
) {
    for segment in segments {
        commands.push(HostPaintCommand::quad(
            segment.clone(),
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn local_rect(origin: &FrameRect, x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x: origin.x + x,
        y: origin.y + y,
        width,
        height,
    }
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
        danger: menu_item_has_flag(item, "danger"),
        ..WorkbenchPopupRowState::default()
    })
}

fn menu_item_has_flag(item: &TemplatePaneMenuItemData, expected: &str) -> bool {
    menu_item_flags(item).any(|flag| flag.eq_ignore_ascii_case(expected))
}

fn menu_item_flag_value(item: &TemplatePaneMenuItemData, expected_key: &str) -> Option<String> {
    menu_item_flags(item).find_map(|flag| {
        let (key, value) = flag.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case(expected_key)
            .then(|| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn menu_item_flags(item: &TemplatePaneMenuItemData) -> impl Iterator<Item = &str> {
    item.raw
        .as_str()
        .split('|')
        .nth(1)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|flag| !flag.is_empty())
}

#[cfg(test)]
mod tests {
    use super::super::super::data::{
        TemplateNodeFrameData, TemplatePaneMenuItemData, TemplatePaneNodeData,
        TemplatePaneOptionData,
    };
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;

    #[test]
    fn menu_item_adornment_kind_reads_icon_danger_and_submenu_flags() {
        let delete = menu_item("Delete|danger,icon=trash", false, false, false);
        let more = menu_item("More Tools|submenu", false, false, false);
        let save = menu_item("Save", false, false, false);

        assert!(menu_item_has_flag(&delete, "danger"));
        assert_eq!(
            menu_item_flag_value(&delete, "icon").as_deref(),
            Some("trash")
        );
        assert_eq!(
            menu_row_adornment_kind(&delete),
            Some(PopupRowAdornmentKind::Trash)
        );
        assert_eq!(
            menu_row_adornment_kind(&more),
            Some(PopupRowAdornmentKind::Chevron)
        );
        assert_eq!(
            menu_row_adornment_kind(&save),
            Some(PopupRowAdornmentKind::Save)
        );
        assert_eq!(popup_menu_row_style(&delete).text, POPUP_ROW_DANGER_TEXT);
        assert_eq!(
            popup_menu_row_style(&delete).adornment,
            POPUP_ROW_DANGER_TEXT
        );
    }

    #[test]
    fn popup_row_style_selector_resolves_state_priority_for_options_and_menu_items() {
        let disabled_pressed = TemplatePaneOptionData {
            pressed: true,
            ..option("disabled", false, false, false, true)
        };
        let focused_selected = option("selected", true, false, false, false);
        let checked_pressed = TemplatePaneMenuItemData {
            pressed: true,
            ..menu_item("Checked", true, false, false)
        };

        let disabled = popup_option_row_style(&disabled_pressed);
        assert_eq!(
            disabled.state,
            zircon_runtime_interface::ui::style::UiPainterResolvedState::Disabled
        );
        assert_eq!(disabled.background, None);
        assert_eq!(disabled.selection_mark, None);
        assert_eq!(disabled.text, PALETTE.text_disabled);

        let focused = popup_option_row_style(&TemplatePaneOptionData {
            focused: true,
            ..focused_selected
        });
        assert_eq!(
            focused.state,
            zircon_runtime_interface::ui::style::UiPainterResolvedState::Focused
        );
        assert_eq!(focused.background, Some(PALETTE.surface_selected));
        assert_eq!(focused.selection_mark, Some(PALETTE.focus_ring));
        assert_eq!(focused.text, PALETTE.focus_ring);

        let checked = popup_menu_row_style(&checked_pressed);
        assert_eq!(
            checked.state,
            zircon_runtime_interface::ui::style::UiPainterResolvedState::Pressed
        );
        assert_eq!(checked.background, Some(PALETTE.surface_selected));
        assert_eq!(checked.selection_mark, Some(PALETTE.focus_ring));
        assert_eq!(checked.adornment, PALETTE.focus_ring);
    }

    #[test]
    fn open_popup_menu_paints_right_aligned_item_icons() {
        let bytes = paint_template_nodes_for_test(180, 180, model_rc(vec![popup_menu_node()]));

        assert!(changed_pixel_count(&bytes, 180, 112, 16, 24, 24) > 0);
        assert_eq!(pixel_at(&bytes, 180, 119, 113), POPUP_ROW_DANGER_TEXT);
        assert!(changed_pixel_count(&bytes, 180, 112, 136, 24, 24) > 0);
    }

    #[test]
    fn selected_dropdown_option_paints_right_check_adornment() {
        let bytes = paint_template_nodes_for_test(150, 120, model_rc(vec![dropdown_node()]));

        assert!(changed_pixel_count(&bytes, 150, 96, 50, 22, 22) > 0);
    }

    fn popup_menu_node() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: "WorkbenchPopupMenu".into(),
            role: "Menu".into(),
            component_role: "menu".into(),
            popup_open: true,
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 10.0,
                width: 130.0,
                height: 150.0,
            },
            structured_menu_items: model_rc(vec![
                menu_item("New|icon=plus", false, false, false),
                menu_item("Open|icon=folder", false, false, false),
                menu_item("Save|icon=save", false, false, false),
                menu_item("Delete|danger,hovered,icon=trash", false, false, true),
                menu_item("More Tools|submenu", false, false, false),
            ]),
            ..TemplatePaneNodeData::default()
        }
    }

    fn dropdown_node() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: "WorkbenchInputDropdown".into(),
            role: "Dropdown".into(),
            component_role: "dropdown".into(),
            popup_open: true,
            frame: TemplateNodeFrameData {
                x: 12.0,
                y: 12.0,
                width: 112.0,
                height: 28.0,
            },
            structured_options: model_rc(vec![
                option("selected", true, false, false, false),
                option("disabled", false, false, false, true),
            ]),
            ..TemplatePaneNodeData::default()
        }
    }

    fn menu_item(
        raw: &str,
        checked: bool,
        separator: bool,
        hovered: bool,
    ) -> TemplatePaneMenuItemData {
        let label = raw.split('|').next().unwrap_or_default();
        TemplatePaneMenuItemData {
            raw: raw.into(),
            action_id: label.into(),
            label: label.into(),
            checked,
            separator,
            disabled: separator,
            hovered,
            ..TemplatePaneMenuItemData::default()
        }
    }

    fn option(
        id: &str,
        selected: bool,
        hovered: bool,
        special: bool,
        disabled: bool,
    ) -> TemplatePaneOptionData {
        TemplatePaneOptionData {
            id: id.into(),
            label: id.into(),
            selected,
            hovered,
            special,
            disabled,
            ..TemplatePaneOptionData::default()
        }
    }

    fn changed_pixel_count(
        bytes: &[u8],
        frame_width: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> usize {
        let mut changed = 0;
        for py in y..(y + height) {
            for px in x..(x + width) {
                let index = ((py as usize * frame_width as usize) + px as usize) * 4;
                if bytes[index..index + 4] != [0, 0, 0, 255] {
                    changed += 1;
                }
            }
        }
        changed
    }

    fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
        let index = ((y as usize * frame_width as usize) + x as usize) * 4;
        [
            bytes[index],
            bytes[index + 1],
            bytes[index + 2],
            bytes[index + 3],
        ]
    }
}
