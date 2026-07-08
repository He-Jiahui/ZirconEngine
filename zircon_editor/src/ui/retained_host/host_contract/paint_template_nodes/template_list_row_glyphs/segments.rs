use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;

const GLYPH_GRID_UNITS: f32 = 16.0;

#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct GlyphSegmentSpec {
    x_units: u8,
    y_units: u8,
    width_units: u8,
    height_units: u8,
}

impl GlyphSegmentSpec {
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
    rect: &FrameRect,
    segments: &[GlyphSegmentSpec],
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for segment in segments {
        commands.push(HostPaintCommand::quad(
            segment_rect(rect, *segment),
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

fn segment_rect(rect: &FrameRect, segment: GlyphSegmentSpec) -> FrameRect {
    let unit_width = rect.width / GLYPH_GRID_UNITS;
    let unit_height = rect.height / GLYPH_GRID_UNITS;
    FrameRect {
        x: rect.x + f32::from(segment.x_units) * unit_width,
        y: rect.y + f32::from(segment.y_units) * unit_height,
        width: f32::from(segment.width_units) * unit_width,
        height: f32::from(segment.height_units) * unit_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_rect_scales_from_sixteen_unit_grid() {
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 32.0,
            height: 16.0,
        };

        let segment = segment_rect(&rect, GlyphSegmentSpec::new(2, 4, 3, 6));

        assert_eq!(segment.x, 14.0);
        assert_eq!(segment.y, 24.0);
        assert_eq!(segment.width, 6.0);
        assert_eq!(segment.height, 6.0);
    }
}
