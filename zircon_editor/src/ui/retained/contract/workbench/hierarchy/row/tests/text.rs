use super::{row_text_x, row_text_y};
use crate::ui::retained_host::host_contract::{data::FrameRect, paint_theme::METRICS};

#[test]
fn hierarchy_row_text_offsets_use_host_metrics_and_preserve_depth_budget() {
    let row = FrameRect {
        x: 4.0,
        y: 8.0,
        width: 60.0,
        height: 18.0,
    };

    assert_eq!(row_text_x(&row, 0, METRICS), 12.0);
    assert_eq!(row_text_y(&row, METRICS), 12.0);
    assert_eq!(row_text_x(&row, 99, METRICS), 42.0);
}
