use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::metrics::AxisLabelMetrics;

pub(super) struct ScaleLinkGeometry {
    pub lobes: [FrameRect; 2],
    pub connector: FrameRect,
}

pub(super) fn scale_link_geometry(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    metrics: &AxisLabelMetrics,
) -> ScaleLinkGeometry {
    let (start_x, start_y) = scale_link_origin_with_metrics(node, rect, metrics);
    ScaleLinkGeometry {
        lobes: [
            FrameRect {
                x: start_x,
                y: start_y,
                width: metrics.link_lobe_width,
                height: metrics.link_lobe_height,
            },
            FrameRect {
                x: start_x + metrics.link_lobe_width - metrics.link_overlap,
                y: start_y,
                width: metrics.link_lobe_width,
                height: metrics.link_lobe_height,
            },
        ],
        connector: FrameRect {
            x: start_x + metrics.link_lobe_width - metrics.link_overlap
                + metrics.link_connector_width,
            y: start_y + metrics.link_lobe_height * 0.5,
            width: metrics.link_overlap,
            height: metrics.link_connector_width,
        },
    }
}

pub(super) fn scale_link_origin_with_metrics(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    metrics: &AxisLabelMetrics,
) -> (f32, f32) {
    let total_width = metrics.link_lobe_width * 2.0 - metrics.link_overlap;
    (
        rect.x + (rect.width - total_width).max(0.0) * 0.5 + node.layout_offset_x,
        rect.y + (rect.height - metrics.link_lobe_height).max(0.0) * 0.5 + node.layout_offset_y,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics() -> AxisLabelMetrics {
        AxisLabelMetrics {
            font_size: 11.0,
            line_height: 13.2,
            link_lobe_width: 6.0,
            link_lobe_height: 7.0,
            link_lobe_radius: 3.0,
            link_overlap: 2.0,
            link_connector_width: 1.0,
        }
    }

    #[test]
    fn scale_link_geometry_centers_lobes_and_connector_from_metrics() {
        let node = TemplatePaneNodeData::default();
        let rect = FrameRect {
            x: 8.0,
            y: 8.0,
            width: 18.0,
            height: 24.0,
        };
        let geometry = scale_link_geometry(&node, &rect, &metrics());

        assert_eq!(geometry.lobes[0].x, 12.0);
        assert_eq!(geometry.lobes[0].y, 16.5);
        assert_eq!(geometry.lobes[1].x, 16.0);
        assert_eq!(geometry.connector.x, 17.0);
        assert_eq!(geometry.connector.y, 20.0);
        assert_eq!(geometry.connector.width, 2.0);
        assert_eq!(geometry.connector.height, 1.0);
    }
}
