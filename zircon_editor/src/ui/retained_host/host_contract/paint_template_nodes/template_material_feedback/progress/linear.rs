use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_style::template_corner_radius;
use super::super::metrics::{linear_progress_radius, material_feedback_metrics};
use super::super::state::{
    progress_fill_color, progress_is_indeterminate, progress_percent, progress_track_color,
};

const PROGRESS_SEGMENT_RATIO_UNITS: f32 = 100.0;

#[derive(Clone, Copy)]
struct IndeterminateProgressSegmentSpec {
    x_units: u8,
    width_units: u8,
}

impl IndeterminateProgressSegmentSpec {
    const fn new(x_units: u8, width_units: u8) -> Self {
        Self {
            x_units,
            width_units,
        }
    }
}

const INDETERMINATE_SEGMENTS: [IndeterminateProgressSegmentSpec; 2] = [
    IndeterminateProgressSegmentSpec::new(12, 36),
    IndeterminateProgressSegmentSpec::new(62, 24),
];

pub(super) fn push_linear_progress_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = linear_progress_radius(
        template_corner_radius(node),
        rect.height,
        material_feedback_metrics(),
    );
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(progress_track_color(node)),
        None,
        0.0,
        radius,
        opacity,
    ));

    let fill = progress_fill_color(node);
    if progress_is_indeterminate(node) {
        for segment in INDETERMINATE_SEGMENTS {
            let bar = indeterminate_segment_rect(rect, segment);
            commands.push(HostPaintCommand::quad(
                bar,
                Some(clip.clone()),
                order + 1,
                Some(fill),
                None,
                0.0,
                radius,
                opacity,
            ));
        }
        return;
    }

    let width = rect.width * progress_percent(node);
    if width <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: width.max(1.0),
            height: rect.height,
        },
        Some(clip.clone()),
        order + 1,
        Some(fill),
        None,
        0.0,
        radius,
        opacity,
    ));
}

fn indeterminate_segment_rect(
    rect: &FrameRect,
    segment: IndeterminateProgressSegmentSpec,
) -> FrameRect {
    FrameRect {
        x: rect.x + rect.width * f32::from(segment.x_units) / PROGRESS_SEGMENT_RATIO_UNITS,
        y: rect.y,
        width: (rect.width * f32::from(segment.width_units) / PROGRESS_SEGMENT_RATIO_UNITS)
            .max(1.0),
        height: rect.height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indeterminate_segment_rect_projects_percent_units() {
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 200.0,
            height: 6.0,
        };

        let first = indeterminate_segment_rect(&rect, INDETERMINATE_SEGMENTS[0]);
        let second = indeterminate_segment_rect(&rect, INDETERMINATE_SEGMENTS[1]);

        assert_eq!(first.x, 34.0);
        assert_eq!(first.y, 20.0);
        assert_eq!(first.width, 72.0);
        assert_eq!(first.height, 6.0);
        assert_eq!(second.x, 134.0);
        assert_eq!(second.width, 48.0);
    }

    #[test]
    fn indeterminate_segment_rect_keeps_minimum_width() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 2.0,
            height: 4.0,
        };

        let segment = indeterminate_segment_rect(&rect, INDETERMINATE_SEGMENTS[1]);

        assert_eq!(segment.width, 1.0);
    }
}
