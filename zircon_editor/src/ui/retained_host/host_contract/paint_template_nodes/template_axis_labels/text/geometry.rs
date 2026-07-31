use super::super::super::super::data::FrameRect;
use super::super::metrics::AxisLabelMetrics;

pub(super) fn axis_label_text_rect(rect: &FrameRect, metrics: &AxisLabelMetrics) -> FrameRect {
    let line_height = metrics.line_height.min(rect.height.max(0.0));
    FrameRect {
        x: rect.x,
        y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
        width: rect.width.max(0.0),
        height: line_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_label_text_rect_stays_inside_an_empty_or_short_slot() {
        let rect = FrameRect {
            x: 4.0,
            y: 10.0,
            width: 0.0,
            height: 24.0,
        };
        let metrics = AxisLabelMetrics {
            font_size: 11.0,
            line_height: 14.0,
            link_lobe_width: 6.0,
            link_lobe_height: 7.0,
            link_lobe_radius: 3.0,
            link_overlap: 2.0,
            link_connector_width: 1.0,
        };

        assert_eq!(
            axis_label_text_rect(&rect, &metrics),
            FrameRect {
                x: 4.0,
                y: 15.0,
                width: 0.0,
                height: 14.0,
            }
        );

        let short = FrameRect {
            x: 4.0,
            y: 10.0,
            width: 12.0,
            height: 6.0,
        };
        assert_eq!(
            axis_label_text_rect(&short, &metrics),
            FrameRect {
                x: 4.0,
                y: 10.0,
                width: 12.0,
                height: 6.0,
            }
        );
    }
}
