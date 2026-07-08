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

fn segment_rect(origin: &FrameRect, segment: DropdownGlyphSegmentSpec) -> FrameRect {
    let unit_width = origin.width / DROPDOWN_SEGMENT_GRID_UNITS;
    let unit_height = origin.height / DROPDOWN_SEGMENT_GRID_UNITS;
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
}
