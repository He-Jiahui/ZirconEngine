use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const ALERT_ICON_SIZE: f32 =
    18.0;
const ALERT_GLYPH_GRID_UNITS: f32 = ALERT_ICON_SIZE;

#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct AlertGlyphSegmentSpec {
    x_units: u8,
    y_units: u8,
    width_units: u8,
    height_units: u8,
}

impl AlertGlyphSegmentSpec {
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const fn alert_segment(
    x_units: u8,
    y_units: u8,
    width_units: u8,
    height_units: u8,
) -> AlertGlyphSegmentSpec {
    AlertGlyphSegmentSpec::new(x_units, y_units, width_units, height_units)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_segments(
    commands: &mut Vec<HostPaintCommand>,
    origin: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[AlertGlyphSegmentSpec],
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

fn segment_rect(origin: &FrameRect, segment: AlertGlyphSegmentSpec) -> FrameRect {
    let scale_x = origin.width / ALERT_GLYPH_GRID_UNITS;
    let scale_y = origin.height / ALERT_GLYPH_GRID_UNITS;
    FrameRect {
        x: origin.x + f32::from(segment.x_units) * scale_x,
        y: origin.y + f32::from(segment.y_units) * scale_y,
        width: (f32::from(segment.width_units) * scale_x).max(1.0),
        height: (f32::from(segment.height_units) * scale_y).max(1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_rect_scales_from_alert_icon_grid() {
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 36.0,
            height: 18.0,
        };

        let segment = segment_rect(&rect, AlertGlyphSegmentSpec::new(8, 14, 2, 2));

        assert_eq!(segment.x, 26.0);
        assert_eq!(segment.y, 34.0);
        assert_eq!(segment.width, 4.0);
        assert_eq!(segment.height, 2.0);
    }
}
