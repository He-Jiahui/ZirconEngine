use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;

const CHIP_SEGMENT_GRID_UNITS: f32 = 12.0;

#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct ChipGlyphSegmentSpec {
    x_units: u8,
    y_units: u8,
    width_units: u8,
    height_units: u8,
}

impl ChipGlyphSegmentSpec {
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
    segments: &[ChipGlyphSegmentSpec],
) {
    for segment in segments {
        let rect = segment_rect(origin, *segment);
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

fn segment_rect(origin: &FrameRect, segment: ChipGlyphSegmentSpec) -> FrameRect {
    let unit_width = origin.width / CHIP_SEGMENT_GRID_UNITS;
    let unit_height = origin.height / CHIP_SEGMENT_GRID_UNITS;
    FrameRect {
        x: origin.x + f32::from(segment.x_units) * unit_width,
        y: origin.y + f32::from(segment.y_units) * unit_height,
        width: (f32::from(segment.width_units) * unit_width).max(0.0),
        height: (f32::from(segment.height_units) * unit_height).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_rect_scales_from_chip_chevron_grid() {
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 24.0,
            height: 12.0,
        };

        let segment = segment_rect(&rect, ChipGlyphSegmentSpec::new(3, 4, 2, 2));

        assert_eq!(segment.x, 16.0);
        assert_eq!(segment.y, 24.0);
        assert_eq!(segment.width, 4.0);
        assert_eq!(segment.height, 2.0);
    }
}
