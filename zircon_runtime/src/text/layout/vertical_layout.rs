use crate::text::layout_geometry::{finite_f32_or_geometry, finite_geometry};

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
    let frame_x = finite_coordinate(frame_x);
    let frame_right_exact = f64::from(frame_x) + f64::from(frame_width);
    let frame_right_candidate = frame_x + frame_width;
    let frame_right_overflowed = !frame_right_candidate.is_finite();
    let frame_right = finite_f32_or_geometry(frame_right_candidate, frame_right_exact);
    let frame_y = finite_coordinate(frame_y);
    let mut frames = Vec::with_capacity(column_heights.len());
    let mut measured_height = 0.0_f32;
    for (index, height) in column_heights.iter().copied().enumerate() {
        let height = finite_non_negative(height);
        measured_height = measured_height.max(height);
        let column_index = (index + 1) as f32;
        let x_candidate = frame_right - column_index * column_advance;
        let x_exact = frame_right_exact - (index + 1) as f64 * f64::from(column_advance);
        frames.push(VerticalColumnFrame {
            x: if frame_right_overflowed {
                finite_geometry(x_exact)
            } else {
                finite_f32_or_geometry(x_candidate, x_exact)
            },
            y: frame_y,
            width: column_width,
            height,
        });
    }

    VerticalColumnLayout {
        column_capacity,
        measured_width: finite_f32_or_geometry(
            frames.len() as f32 * column_advance,
            frames.len() as f64 * f64::from(column_advance),
        ),
        measured_height,
        frames,
    }
}

#[cfg(test)]
#[path = "vertical_layout/tests/single_pass_frame_tests.rs"]
mod single_pass_frame_tests;

fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
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
#[path = "tests/vertical_layout.rs"]
mod tests;
