use super::super::super::data::FrameRect;
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;

const STEPPER_GLYPH_GRID_WIDTH_UNITS: f32 = 50.0;
const STEPPER_GLYPH_GRID_HEIGHT_UNITS: f32 = 80.0;

#[derive(Clone, Copy)]
struct FieldStepperGlyphSegmentSpec {
    x_units: u8,
    y_units: u8,
    width_units: u8,
    height_units: u8,
}

impl FieldStepperGlyphSegmentSpec {
    const fn new(x_units: u8, y_units: u8, width_units: u8, height_units: u8) -> Self {
        Self {
            x_units,
            y_units,
            width_units,
            height_units,
        }
    }
}

const STEPPER_GLYPH_SEGMENTS: &[FieldStepperGlyphSegmentSpec] = &[
    FieldStepperGlyphSegmentSpec::new(20, 10, 10, 10),
    FieldStepperGlyphSegmentSpec::new(10, 20, 30, 7),
    FieldStepperGlyphSegmentSpec::new(10, 55, 30, 7),
    FieldStepperGlyphSegmentSpec::new(20, 65, 10, 10),
];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_stepper_glyph_segments(
    commands: &mut Vec<HostPaintCommand>,
    origin: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for segment in STEPPER_GLYPH_SEGMENTS {
        let rect = segment_rect(origin, *segment);
        if intersect(&rect, clip).is_none() {
            continue;
        }
        commands.push(HostPaintCommand::quad(
            rect,
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

fn segment_rect(origin: &FrameRect, segment: FieldStepperGlyphSegmentSpec) -> FrameRect {
    let unit_width = origin.width / STEPPER_GLYPH_GRID_WIDTH_UNITS;
    let unit_height = origin.height / STEPPER_GLYPH_GRID_HEIGHT_UNITS;
    FrameRect {
        x: origin.x + f32::from(segment.x_units) * unit_width,
        y: origin.y + f32::from(segment.y_units) * unit_height,
        width: (f32::from(segment.width_units) * unit_width).max(1.0),
        height: (f32::from(segment.height_units) * unit_height).max(1.0),
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
    fn segment_rect_scales_from_stepper_glyph_grid() {
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 20.0,
            height: 16.0,
        };

        let segment = segment_rect(&rect, FieldStepperGlyphSegmentSpec::new(20, 10, 10, 10));

        assert_close(segment.x, 18.0);
        assert_close(segment.y, 22.0);
        assert_close(segment.width, 4.0);
        assert_close(segment.height, 2.0);
    }

    #[test]
    fn segment_rect_preserves_stepper_subpixel_stroke_height() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 16.0,
        };

        let segment = segment_rect(&rect, FieldStepperGlyphSegmentSpec::new(10, 20, 30, 7));

        assert_close(segment.x, 2.0);
        assert_close(segment.y, 4.0);
        assert_close(segment.width, 6.0);
        assert_close(segment.height, 1.4);
    }

    #[test]
    fn stepper_glyph_segments_skip_fully_clipped_stepper() {
        let origin = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 20.0,
            height: 16.0,
        };
        let clip = FrameRect {
            x: 40.0,
            y: 0.0,
            width: 40.0,
            height: 80.0,
        };
        let mut commands = Vec::new();

        push_stepper_glyph_segments(&mut commands, &origin, &clip, 2, [255, 255, 255, 255], 1.0);

        assert!(commands.is_empty());
    }

    #[test]
    fn stepper_glyph_segments_keep_partially_visible_segments() {
        let origin = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 20.0,
            height: 16.0,
        };
        let clip = FrameRect {
            x: 14.0,
            y: 22.0,
            width: 6.0,
            height: 8.0,
        };
        let mut commands = Vec::new();

        push_stepper_glyph_segments(&mut commands, &origin, &clip, 2, [255, 255, 255, 255], 1.0);

        assert!(!commands.is_empty());
        assert!(commands
            .iter()
            .all(|command| command.clip_frame.as_ref() == Some(&clip)));
    }
}
