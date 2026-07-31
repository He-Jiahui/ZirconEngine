use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;

const DROPDOWN_SEGMENT_GRID_UNITS: f32 = 14.0;

#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct DropdownGlyphSegmentSpec
{
    x_units: u8,
    y_units: u8,
    width_units: u8,
    height_units: u8,
}

impl DropdownGlyphSegmentSpec {
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
    segments: &[DropdownGlyphSegmentSpec],
) {
    if !has_paintable_rect(origin) {
        return;
    }
    for segment in segments {
        let rect = segment_rect(origin, *segment);
        if !has_paintable_rect(&rect) {
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

fn segment_rect(origin: &FrameRect, segment: DropdownGlyphSegmentSpec) -> FrameRect {
    let unit_width = origin.width / DROPDOWN_SEGMENT_GRID_UNITS;
    let unit_height = origin.height / DROPDOWN_SEGMENT_GRID_UNITS;
    FrameRect {
        x: origin.x + f32::from(segment.x_units) * unit_width,
        y: origin.y + f32::from(segment.y_units) * unit_height,
        width: f32::from(segment.width_units) * unit_width,
        height: f32::from(segment.height_units) * unit_height,
    }
}

fn has_paintable_rect(rect: &FrameRect) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_rect_scales_from_dropdown_chevron_grid() {
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 28.0,
            height: 14.0,
        };

        let segment = segment_rect(&rect, DropdownGlyphSegmentSpec::new(3, 5, 2, 2));

        assert_eq!(segment.x, 16.0);
        assert_eq!(segment.y, 25.0);
        assert_eq!(segment.width, 4.0);
        assert_eq!(segment.height, 2.0);
    }

    #[test]
    fn subpixel_segment_stays_within_the_original_glyph_slot() {
        let origin = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 0.7,
            height: 0.7,
        };

        let segment = segment_rect(&origin, DropdownGlyphSegmentSpec::new(3, 5, 2, 2));

        assert!(segment.width > 0.0 && segment.width < 1.0);
        assert!(segment.height > 0.0 && segment.height < 1.0);
        assert!(segment.x >= origin.x);
        assert!(segment.y >= origin.y);
        assert!(segment.x + segment.width <= origin.x + origin.width);
        assert!(segment.y + segment.height <= origin.y + origin.height);
    }

    #[test]
    fn empty_glyph_slot_does_not_emit_segment_commands() {
        let origin = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 0.0,
            height: 12.0,
        };
        let mut commands = Vec::new();

        push_segments(
            &mut commands,
            &origin,
            &origin,
            0,
            [255, 255, 255, 255],
            1.0,
            &[DropdownGlyphSegmentSpec::new(3, 5, 2, 2)],
        );

        assert!(commands.is_empty());
    }
}
