use super::constants::{ROW_GAP, ROW_HEIGHT, ROW_Y};

pub(super) fn content_height(item_count: usize) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        ROW_Y + item_count as f32 * ROW_HEIGHT + (item_count as f32 - 1.0) * ROW_GAP + ROW_Y
    }
}
