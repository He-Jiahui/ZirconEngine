#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct VerticalColumnFrame {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct VerticalColumnLayout {
    pub(crate) column_capacity: usize,
    pub(crate) frames: Vec<VerticalColumnFrame>,
    pub(crate) measured_width: f32,
    pub(crate) measured_height: f32,
}

pub(crate) fn layout_vertical_rl_columns(
    frame_x: f32,
    frame_y: f32,
    frame_width: f32,
    column_width: f32,
    column_advance: f32,
    column_heights: &[f32],
) -> VerticalColumnLayout {
    let column_width = finite_non_negative(column_width);
    let column_advance = finite_positive(column_advance).unwrap_or(column_width.max(1.0));
    let frame_width = finite_non_negative(frame_width);
    let column_capacity = (frame_width.max(column_advance) / column_advance)
        .floor()
        .max(1.0) as usize;
    let frame_right = finite_coordinate(frame_x) + frame_width;
    let frame_y = finite_coordinate(frame_y);
    let mut frames = Vec::with_capacity(column_heights.len());
    let mut measured_height = 0.0_f32;
    for (index, height) in column_heights.iter().copied().enumerate() {
        let height = finite_non_negative(height);
        measured_height = measured_height.max(height);
        frames.push(VerticalColumnFrame {
            x: frame_right - (index + 1) as f32 * column_advance,
            y: frame_y,
            width: column_width,
            height,
        });
    }

    VerticalColumnLayout {
        column_capacity,
        measured_width: frames.len() as f32 * column_advance,
        measured_height,
        frames,
    }
}

#[cfg(test)]
#[path = "vertical_layout/single_pass_frame_tests.rs"]
mod single_pass_frame_tests;

fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_positive(value: f32) -> Option<f32> {
    (value.is_finite() && value > 0.0).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::layout_vertical_rl_columns;

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

    #[test]
    fn vertical_rl_layout_reports_cross_and_main_axis_extents() {
        let layout = layout_vertical_rl_columns(0.0, 0.0, 10.0, 16.0, 20.0, &[32.0, 48.0]);

        assert_eq!(layout.column_capacity, 1);
        assert_eq!(layout.measured_width, 40.0);
        assert_eq!(layout.measured_height, 48.0);
    }
}
