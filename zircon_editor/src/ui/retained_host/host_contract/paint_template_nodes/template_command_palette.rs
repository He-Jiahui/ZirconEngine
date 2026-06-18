use super::super::data::{FrameRect, TemplatePaneNodeData, TemplatePaneOptionData};
use super::super::paint_geometry::intersect;
use super::super::paint_theme::PALETTE;
use super::render_commands::HostPaintCommand;
use super::style_selector::{select_workbench_popup_row_style, WorkbenchPopupRowState};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const PANEL_RADIUS: f32 = 6.0;
const PANEL_PADDING_X: f32 = 12.0;
const SEARCH_TOP: f32 = 10.0;
const SEARCH_HEIGHT: f32 = 30.0;
const SEARCH_TEXT_X: f32 = 10.0;
const SEARCH_TEXT_Y: f32 = 7.0;
const LIST_TOP: f32 = 48.0;
const ROW_INSET_X: f32 = 8.0;
const ROW_HEIGHT: f32 = 26.0;
const ROW_TEXT_X: f32 = 9.0;
const ROW_TEXT_Y: f32 = 5.0;
const ROW_SELECTED_MARK_WIDTH: f32 = 3.0;
const EMPTY_TEXT_Y: f32 = 58.0;
const FONT_SIZE: f32 = 12.0;
const LINE_HEIGHT: f32 = 14.4;

pub(super) fn push_command_palette_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_command_palette(node) {
        return false;
    }
    if !node.popup_open {
        return true;
    }

    let rect = pixel_aligned_rect(rect);
    if rect.width <= 1.0 || rect.height <= 1.0 {
        return true;
    }

    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(PALETTE.popup),
        Some(PALETTE.border),
        1.0,
        PANEL_RADIUS,
        opacity,
    ));

    let search_rect = FrameRect {
        x: rect.x + PANEL_PADDING_X,
        y: rect.y + SEARCH_TOP,
        width: (rect.width - PANEL_PADDING_X * 2.0).max(1.0),
        height: SEARCH_HEIGHT,
    };
    commands.push(HostPaintCommand::quad(
        search_rect.clone(),
        Some(clip.clone()),
        order + 1,
        Some(PALETTE.surface_inset),
        Some(PALETTE.focus_ring),
        1.0,
        4.0,
        opacity,
    ));

    let query = node.search_query.as_str();
    let (search_text, search_color) = if query.trim().is_empty() {
        ("Search commands", PALETTE.text_muted)
    } else {
        (query, PALETTE.text)
    };
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: search_rect.x + SEARCH_TEXT_X,
            y: search_rect.y + SEARCH_TEXT_Y,
            width: (search_rect.width - SEARCH_TEXT_X * 2.0).max(1.0),
            height: LINE_HEIGHT,
        },
        Some(clip.clone()),
        order + 2,
        search_text.to_string(),
        search_color,
        FONT_SIZE,
        LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));

    let row_count = node.structured_options.row_count();
    if row_count == 0 {
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: rect.x + PANEL_PADDING_X,
                y: rect.y + EMPTY_TEXT_Y,
                width: (rect.width - PANEL_PADDING_X * 2.0).max(1.0),
                height: LINE_HEIGHT,
            },
            Some(clip.clone()),
            order + 3,
            "No commands found".to_string(),
            PALETTE.text_muted,
            FONT_SIZE,
            LINE_HEIGHT,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
        return true;
    }

    for row in 0..row_count {
        let Some(option) = node.structured_options.row_data(row) else {
            continue;
        };
        let row_rect = FrameRect {
            x: rect.x + ROW_INSET_X,
            y: rect.y + LIST_TOP + row as f32 * ROW_HEIGHT,
            width: (rect.width - ROW_INSET_X * 2.0).max(1.0),
            height: ROW_HEIGHT,
        };
        let style = command_row_style(&option);
        push_command_row_surface(
            commands,
            &row_rect,
            clip,
            order + 4 + row as i32 * 3,
            style,
            opacity,
        );
        push_command_row_label(
            commands,
            &row_rect,
            clip,
            order + 6 + row as i32 * 3,
            option.label.to_string(),
            style.text,
            opacity,
        );
    }

    true
}

fn is_command_palette(node: &TemplatePaneNodeData) -> bool {
    node.role.as_str() == "CommandPalette" || node.component_role.as_str() == "command-palette"
}

fn command_row_style(
    option: &TemplatePaneOptionData,
) -> super::style_selector::WorkbenchPopupRowStyle {
    select_workbench_popup_row_style(WorkbenchPopupRowState {
        focused: option.focused,
        disabled: option.disabled,
        selected: option.selected || option.special,
        ..WorkbenchPopupRowState::default()
    })
}

fn push_command_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    style: super::style_selector::WorkbenchPopupRowStyle,
    opacity: f32,
) {
    if intersect(row_rect, clip).is_none() {
        return;
    }
    if let Some(background) = style.background {
        commands.push(HostPaintCommand::quad(
            row_rect.clone(),
            Some(clip.clone()),
            order,
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
                width: ROW_SELECTED_MARK_WIDTH,
                height: (row_rect.height - 8.0).max(1.0),
            },
            Some(clip.clone()),
            order + 1,
            Some(selection_mark),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn push_command_row_label(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    text: String,
    color: [u8; 4],
    opacity: f32,
) {
    if text.is_empty() || intersect(row_rect, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: row_rect.x + ROW_TEXT_X,
            y: row_rect.y + ROW_TEXT_Y,
            width: (row_rect.width - ROW_TEXT_X * 2.0).max(1.0),
            height: (row_rect.height - ROW_TEXT_Y * 2.0).max(12.0),
        },
        Some(clip.clone()),
        order,
        text,
        color,
        FONT_SIZE,
        LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}
