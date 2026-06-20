use super::super::data::FrameRect;
use super::metrics::menu_item_row_height;

pub(crate) fn menu_item_row_frame(
    menu_frame: &FrameRect,
    row_count: usize,
    row: usize,
) -> Option<FrameRect> {
    let row_height = menu_item_row_height(menu_frame, row_count)?;
    Some(FrameRect {
        x: menu_frame.x,
        y: menu_frame.y + row as f32 * row_height,
        width: menu_frame.width.max(1.0),
        height: row_height,
    })
}
