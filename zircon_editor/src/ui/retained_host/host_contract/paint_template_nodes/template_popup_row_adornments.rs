use super::super::data::{FrameRect, TemplatePaneMenuItemData};
use super::super::paint_geometry::intersect;
use super::super::paint_theme::PALETTE;
use super::render_commands::HostPaintCommand;

const POPUP_ROW_ADORNMENT_RIGHT: f32 = 12.0;
const POPUP_ROW_ADORNMENT_SIZE: f32 = 14.0;

pub(super) const POPUP_ROW_ADORNMENT_RESERVED_WIDTH: f32 = 30.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PopupRowAdornmentKind {
    Check,
    Chevron,
    Plus,
    Folder,
    Save,
    Trash,
}

pub(super) fn option_adornment_kind(selected: bool) -> Option<PopupRowAdornmentKind> {
    selected.then_some(PopupRowAdornmentKind::Check)
}

pub(super) fn menu_row_adornment_kind(
    item: &TemplatePaneMenuItemData,
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

pub(super) fn push_popup_row_adornment(
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

pub(super) fn menu_item_has_flag(item: &TemplatePaneMenuItemData, expected: &str) -> bool {
    menu_item_flags(item).any(|flag| flag.eq_ignore_ascii_case(expected))
}

pub(super) fn menu_item_flag_value(
    item: &TemplatePaneMenuItemData,
    expected_key: &str,
) -> Option<String> {
    menu_item_flags(item).find_map(|flag| {
        let (key, value) = flag.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case(expected_key)
            .then(|| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
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
