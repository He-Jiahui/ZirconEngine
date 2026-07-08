use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments::{alert_segment as seg, push_segments};
use super::palette::ALERT_GLYPH_DARK;

const WARNING_TRIANGLE_CANONICAL_PIXELS: f32 = 18.0;
const WARNING_TRIANGLE_SUBUNITS_PER_PIXEL: f32 = 20.0;
const WARNING_TRIANGLE_GRID_UNITS: f32 =
    WARNING_TRIANGLE_CANONICAL_PIXELS * WARNING_TRIANGLE_SUBUNITS_PER_PIXEL;
const WARNING_TRIANGLE_CENTER_RATIO: f32 = 0.5;

#[derive(Clone, Copy)]
struct WarningTriangleRowSpec {
    y_units: u16,
    width_units: u16,
    height_units: u16,
}

impl WarningTriangleRowSpec {
    const fn new(y_units: u16, width_units: u16, height_units: u16) -> Self {
        Self {
            y_units,
            width_units,
            height_units,
        }
    }
}

const WARNING_TRIANGLE_ROWS: [WarningTriangleRowSpec; 6] = [
    WarningTriangleRowSpec::new(60, 60, 40),
    WarningTriangleRowSpec::new(97, 100, 40),
    WarningTriangleRowSpec::new(134, 140, 40),
    WarningTriangleRowSpec::new(171, 180, 40),
    WarningTriangleRowSpec::new(208, 220, 40),
    WarningTriangleRowSpec::new(245, 260, 40),
];

pub(super) fn push_warning_mark(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for row in WARNING_TRIANGLE_ROWS {
        commands.push(HostPaintCommand::quad(
            warning_triangle_row_rect(rect, row),
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
    push_segments(
        commands,
        rect,
        clip,
        order + 1,
        ALERT_GLYPH_DARK,
        opacity,
        &[seg(8, 8, 2, 4), seg(8, 14, 2, 2)],
    );
}

fn warning_triangle_row_rect(rect: &FrameRect, row: WarningTriangleRowSpec) -> FrameRect {
    let unit_width = rect.width / WARNING_TRIANGLE_GRID_UNITS;
    let unit_height = rect.height / WARNING_TRIANGLE_GRID_UNITS;
    let width = f32::from(row.width_units) * unit_width;
    FrameRect {
        x: rect.x + rect.width * WARNING_TRIANGLE_CENTER_RATIO
            - width * WARNING_TRIANGLE_CENTER_RATIO,
        y: rect.y + f32::from(row.y_units) * unit_height,
        width: width.max(1.0),
        height: (f32::from(row.height_units) * unit_height).max(1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.001,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn warning_triangle_rows_preserve_default_alert_shape() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: WARNING_TRIANGLE_CANONICAL_PIXELS,
            height: WARNING_TRIANGLE_CANONICAL_PIXELS,
        };

        let first = warning_triangle_row_rect(&rect, WARNING_TRIANGLE_ROWS[0]);
        let last = warning_triangle_row_rect(&rect, WARNING_TRIANGLE_ROWS[5]);

        assert_close(first.x, 7.5);
        assert_close(first.y, 3.0);
        assert_close(first.width, 3.0);
        assert_close(first.height, 2.0);
        assert_close(last.x, 2.5);
        assert_close(last.y, 12.25);
        assert_close(last.width, 13.0);
        assert_close(last.height, 2.0);
    }

    #[test]
    fn warning_triangle_rows_scale_with_alert_rect() {
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 36.0,
            height: 18.0,
        };

        let first = warning_triangle_row_rect(&rect, WARNING_TRIANGLE_ROWS[0]);

        assert_close(first.x, 25.0);
        assert_close(first.y, 23.0);
        assert_close(first.width, 6.0);
        assert_close(first.height, 2.0);
    }
}
