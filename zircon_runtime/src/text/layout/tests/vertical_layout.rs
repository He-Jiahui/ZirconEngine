use super::layout_vertical_rl_columns;

// 固定步进与列宽不同，核对左坐标、原点和首列尺寸。
#[test]
fn vertical_rl_columns_are_placed_from_right_to_left() {
    let layout = layout_vertical_rl_columns(10.0, 4.0, 72.0, 20.0, 24.0, &[50.0, 36.0]);

    assert_eq!(layout.column_capacity, 3);
    assert_eq!(layout.frames[0].x, 58.0);
    assert_eq!(layout.frames[1].x, 34.0);
    assert_eq!(layout.frames[0].y, 4.0);
    assert_eq!(layout.frames[0].width, 20.0);
    assert_eq!(layout.frames[0].height, 50.0);
}

// 容量估计不裁掉第二列；总体宽度按步进累计，高度取最大列高。
#[test]
fn vertical_rl_layout_reports_cross_and_main_axis_extents() {
    let layout = layout_vertical_rl_columns(0.0, 0.0, 10.0, 16.0, 20.0, &[32.0, 48.0]);

    assert_eq!(layout.column_capacity, 1);
    assert_eq!(layout.measured_width, 40.0);
    assert_eq!(layout.measured_height, 48.0);
}
