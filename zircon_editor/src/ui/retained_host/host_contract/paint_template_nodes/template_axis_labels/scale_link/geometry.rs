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
    let dimensions = scale_link_dimensions(rect, metrics);
    let (start_x, start_y) = scale_link_origin_for_dimensions(node, rect, dimensions);
    ScaleLinkGeometry {
        lobes: [
            FrameRect {
                x: start_x,
                y: start_y,
                width: dimensions.lobe_width,
                height: dimensions.lobe_height,
            },
            FrameRect {
                x: start_x + dimensions.lobe_width - dimensions.overlap,
                y: start_y,
                width: dimensions.lobe_width,
                height: dimensions.lobe_height,
            },
        ],
        connector: FrameRect {
            x: start_x + dimensions.lobe_width - dimensions.overlap + dimensions.connector_width,
            y: start_y + dimensions.lobe_height * 0.5,
            width: dimensions.overlap,
            height: dimensions.connector_width,
        },
    }
}

pub(super) fn scale_link_origin_with_metrics(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    metrics: &AxisLabelMetrics,
) -> (f32, f32) {
    scale_link_origin_for_dimensions(node, rect, scale_link_dimensions(rect, metrics))
}

#[derive(Clone, Copy)]
struct ScaleLinkDimensions {
    lobe_width: f32,
    lobe_height: f32,
    overlap: f32,
    connector_width: f32,
}

impl ScaleLinkDimensions {
    fn total_width(self) -> f32 {
        self.lobe_width * 2.0 - self.overlap
    }
}

fn scale_link_dimensions(rect: &FrameRect, metrics: &AxisLabelMetrics) -> ScaleLinkDimensions {
    let total_width = metrics.link_lobe_width * 2.0 - metrics.link_overlap;
    let scale = (rect.width.max(0.0) / total_width.max(1.0))
        .min(rect.height.max(0.0) / metrics.link_lobe_height.max(1.0))
        .min(1.0);
    ScaleLinkDimensions {
        lobe_width: metrics.link_lobe_width * scale,
        lobe_height: metrics.link_lobe_height * scale,
        overlap: metrics.link_overlap * scale,
        connector_width: metrics.link_connector_width * scale,
    }
}

fn scale_link_origin_for_dimensions(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    dimensions: ScaleLinkDimensions,
) -> (f32, f32) {
    (
        rect.x + (rect.width - dimensions.total_width()).max(0.0) * 0.5 + node.layout_offset_x,
        rect.y + (rect.height - dimensions.lobe_height).max(0.0) * 0.5 + node.layout_offset_y,
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

    #[test]
    fn scale_link_geometry_scales_each_part_into_a_narrow_short_slot() {
        let node = TemplatePaneNodeData::default();
        let rect = FrameRect {
            x: 8.0,
            y: 8.0,
            width: 4.0,
            height: 3.0,
        };
        let geometry = scale_link_geometry(&node, &rect, &metrics());

        for part in [
            geometry.lobes[0].clone(),
            geometry.lobes[1].clone(),
            geometry.connector,
        ] {
            assert_contained(part, &rect);
        }
    }

    fn assert_contained(part: FrameRect, parent: &FrameRect) {
        let epsilon = 0.000_1;
        assert!(part.x >= parent.x - epsilon);
        assert!(part.y >= parent.y - epsilon);
        assert!(part.x + part.width <= parent.x + parent.width + epsilon);
        assert!(part.y + part.height <= parent.y + parent.height + epsilon);
    }
}
