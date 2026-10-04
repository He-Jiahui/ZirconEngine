use super::super::super::super::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_geometry::bounded_extent,
};
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

pub(super) fn scale_link_asset_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    metrics: &AxisLabelMetrics,
) -> FrameRect {
    let geometry = scale_link_geometry(node, rect, metrics);
    let natural_left = geometry.lobes[0].x.min(geometry.connector.x);
    let natural_top = geometry.lobes[0].y.min(geometry.connector.y);
    let natural_right = (geometry.lobes[1].x + geometry.lobes[1].width)
        .max(geometry.connector.x + geometry.connector.width);
    let natural_bottom = (geometry.lobes[0].y + geometry.lobes[0].height)
        .max(geometry.connector.y + geometry.connector.height);
    let natural_width = bounded_extent(natural_right - natural_left);
    let natural_height = bounded_extent(natural_bottom - natural_top);
    let natural_size = natural_width
        .max(natural_height)
        .max(bounded_extent(metrics.link_lobe_radius * 2.0));
    let requested_size = if node.layout_icon_size.is_finite() && node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        natural_size
    };
    let available_extent = bounded_extent(rect.width).max(bounded_extent(rect.height));
    let size = bounded_extent(requested_size).min(available_extent);
    FrameRect {
        x: natural_left + (natural_width - size) * 0.5,
        y: natural_top + (natural_height - size) * 0.5,
        width: size,
        height: size,
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
    let lobe_width = bounded_extent(metrics.link_lobe_width);
    let lobe_height = bounded_extent(metrics.link_lobe_height);
    let overlap = bounded_extent(metrics.link_overlap).min(lobe_width * 2.0);
    let total_width = bounded_extent(lobe_width * 2.0 - overlap);
    let available_width = bounded_extent(rect.width);
    let available_height = bounded_extent(rect.height);
    let scale = if total_width <= 0.0 || lobe_height <= 0.0 {
        0.0
    } else {
        (available_width / total_width)
            .min(available_height / lobe_height)
            .min(1.0)
    };
    ScaleLinkDimensions {
        lobe_width: lobe_width * scale,
        lobe_height: lobe_height * scale,
        overlap: overlap * scale,
        connector_width: bounded_extent(metrics.link_connector_width) * scale,
    }
}

fn scale_link_origin_for_dimensions(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    dimensions: ScaleLinkDimensions,
) -> (f32, f32) {
    let available_width = bounded_extent(rect.width);
    let available_height = bounded_extent(rect.height);
    (
        rect.x + (available_width - dimensions.total_width()).max(0.0) * 0.5 + node.layout_offset_x,
        rect.y + (available_height - dimensions.lobe_height).max(0.0) * 0.5 + node.layout_offset_y,
    )
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
