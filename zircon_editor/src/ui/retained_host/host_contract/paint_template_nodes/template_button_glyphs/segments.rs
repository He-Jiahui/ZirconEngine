use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;

const BUTTON_GLYPH_CANONICAL_PIXELS: f32 = 14.0;
const BUTTON_GLYPH_SUBUNITS_PER_PIXEL: f32 = 5.0;
const BUTTON_GLYPH_GRID_UNITS: f32 =
    BUTTON_GLYPH_CANONICAL_PIXELS * BUTTON_GLYPH_SUBUNITS_PER_PIXEL;

#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct ButtonGlyphSegmentSpec
{
    x_units: u8,
    y_units: u8,
    width_units: u8,
    height_units: u8,
}

impl ButtonGlyphSegmentSpec {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const fn new(
        x_units: u8,
        y_units: u8,
        width_units: u8,
        height_units: u8,
    ) -> Self {
        Self {
            x_units,
            y_units,
            width_units,
            height_units,
        }
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_segments(
    commands: &mut Vec<HostPaintCommand>,
    origin: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[ButtonGlyphSegmentSpec],
) {
    for segment in segments {
        commands.push(HostPaintCommand::quad(
            segment_rect(origin, *segment),
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

fn segment_rect(origin: &FrameRect, segment: ButtonGlyphSegmentSpec) -> FrameRect {
    let unit_width = origin.width / BUTTON_GLYPH_GRID_UNITS;
    let unit_height = origin.height / BUTTON_GLYPH_GRID_UNITS;
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
    fn segment_rect_scales_from_button_glyph_grid() {
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 28.0,
            height: 14.0,
        };

        let segment = segment_rect(&rect, ButtonGlyphSegmentSpec::new(30, 10, 10, 50));

        assert_close(segment.x, 22.0);
        assert_close(segment.y, 22.0);
        assert_close(segment.width, 4.0);
        assert_close(segment.height, 10.0);
    }

    #[test]
    fn segment_rect_preserves_subpixel_stroke_units_at_default_size() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: BUTTON_GLYPH_CANONICAL_PIXELS,
            height: BUTTON_GLYPH_CANONICAL_PIXELS,
        };

        let segment = segment_rect(&rect, ButtonGlyphSegmentSpec::new(15, 20, 40, 6));

        assert_close(segment.x, 3.0);
        assert_close(segment.y, 4.0);
        assert_close(segment.width, 8.0);
        assert_close(segment.height, 1.2);
    }
}
