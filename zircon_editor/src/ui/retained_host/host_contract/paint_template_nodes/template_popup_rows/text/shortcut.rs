use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::surface::POPUP_ROW_ORDER_OFFSET;
use super::metrics::{DEFAULT_POPUP_FONT_SIZE, MIN_TEXT_RECT_HEIGHT, POPUP_ROW_TEXT_Y};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_popup_row_shortcut(
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
