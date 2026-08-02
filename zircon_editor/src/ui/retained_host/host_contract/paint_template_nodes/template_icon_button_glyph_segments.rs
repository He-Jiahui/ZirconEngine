use super::super::{data::FrameRect, paint_geometry::bounded_extent};
use super::render_commands::HostPaintCommand;

const ICON_BUTTON_GLYPH_CANONICAL_PIXELS: f32 = 16.0;
const ICON_BUTTON_GLYPH_SUBUNITS_PER_PIXEL: f32 = 20.0;
const ICON_BUTTON_GLYPH_GRID_UNITS: f32 =
    ICON_BUTTON_GLYPH_CANONICAL_PIXELS * ICON_BUTTON_GLYPH_SUBUNITS_PER_PIXEL;

#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct IconButtonGlyphSegmentSpec
{
    x_units: u16,
    y_units: u16,
    width_units: u16,
    height_units: u16,
}

impl IconButtonGlyphSegmentSpec {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const fn new(
        x_units: u16,
        y_units: u16,
        width_units: u16,
        height_units: u16,
    ) -> Self {
        Self {
            x_units,
            y_units,
            width_units,
            height_units,
        }
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const fn icon_button_segment(
    x_units: u16,
    y_units: u16,
    width_units: u16,
    height_units: u16,
) -> IconButtonGlyphSegmentSpec {
    IconButtonGlyphSegmentSpec::new(x_units, y_units, width_units, height_units)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_icon_button_glyph_segments(
    commands: &mut Vec<HostPaintCommand>,
    origin: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[IconButtonGlyphSegmentSpec],
) {
    for segment in segments {
        let rect = icon_button_segment_rect(origin, *segment);
        if rect.width <= 0.0 || rect.height <= 0.0 {
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

fn icon_button_segment_rect(origin: &FrameRect, segment: IconButtonGlyphSegmentSpec) -> FrameRect {
    let scale_x = bounded_extent(origin.width) / ICON_BUTTON_GLYPH_GRID_UNITS;
    let scale_y = bounded_extent(origin.height) / ICON_BUTTON_GLYPH_GRID_UNITS;
    FrameRect {
        x: origin.x + f32::from(segment.x_units) * scale_x,
        y: origin.y + f32::from(segment.y_units) * scale_y,
        width: bounded_extent(f32::from(segment.width_units) * scale_x),
        height: bounded_extent(f32::from(segment.height_units) * scale_y),
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
    fn segment_rect_preserves_fractional_toolbar_icon_grid_units() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: ICON_BUTTON_GLYPH_CANONICAL_PIXELS,
            height: ICON_BUTTON_GLYPH_CANONICAL_PIXELS,
        };

        let segment =
            icon_button_segment_rect(&rect, IconButtonGlyphSegmentSpec::new(153, 109, 22, 88));

        assert_close(segment.x, 7.65);
        assert_close(segment.y, 5.45);
        assert_close(segment.width, 1.1);
        assert_close(segment.height, 4.4);
    }

    #[test]
    fn segment_rect_scales_from_toolbar_icon_grid() {
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 32.0,
            height: 16.0,
        };

        let segment =
            icon_button_segment_rect(&rect, IconButtonGlyphSegmentSpec::new(144, 60, 32, 200));

        assert_close(segment.x, 24.4);
        assert_close(segment.y, 23.0);
        assert_close(segment.width, 3.2);
        assert_close(segment.height, 10.0);
    }

    #[test]
    fn collapsed_icon_origin_emits_no_glyph_segments() {
        let origin = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 0.0,
            height: ICON_BUTTON_GLYPH_CANONICAL_PIXELS,
        };
        let mut commands = Vec::new();

        push_icon_button_glyph_segments(
            &mut commands,
            &origin,
            &origin,
            0,
            [255, 255, 255, 255],
            1.0,
            &[IconButtonGlyphSegmentSpec::new(144, 60, 32, 200)],
        );

        assert!(commands.is_empty());
    }
}
