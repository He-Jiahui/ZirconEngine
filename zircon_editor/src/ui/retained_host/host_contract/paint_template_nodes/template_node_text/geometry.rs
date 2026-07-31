use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_node_images::{is_icon_node, is_icon_only_node, leading_icon_size};
use super::super::template_node_labels::template_node_label;
use super::metrics::{template_node_text_geometry_metrics, TemplateNodeTextGeometryMetrics};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_rect_for_node(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    text_rect_for_node_with_metrics(node, rect, template_node_text_geometry_metrics())
}

fn text_rect_for_node_with_metrics(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    metrics: TemplateNodeTextGeometryMetrics,
) -> FrameRect {
    let frame_width = finite_extent(rect.width);
    let frame_height = finite_extent(rect.height);
    let horizontal = metrics.horizontal_inset.min(frame_width * 0.25).max(0.0);
    let vertical = metrics
        .vertical_inset
        .min(
            ((frame_height - metrics.minimum_text_height) * 0.5)
                .max(metrics.edge_guard)
                .min(frame_height * 0.5),
        )
        .max(0.0);
    let layout_rect = FrameRect {
        x: finite_coordinate(rect.x),
        y: finite_coordinate(rect.y),
        width: frame_width,
        height: frame_height,
    };
    let mut x = layout_rect.x + horizontal;
    let mut width = (frame_width - horizontal * 2.0).max(0.0);
    if is_leading_icon_text_node(node) {
        let leading = (leading_icon_size(&layout_rect) + metrics.horizontal_inset)
            .min(width)
            .max(0.0);
        x += leading;
        width = (width - leading).max(0.0);
    }
    FrameRect {
        x,
        y: layout_rect.y + vertical,
        width,
        height: (frame_height - vertical * 2.0).max(0.0),
    }
}

fn finite_extent(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics() -> TemplateNodeTextGeometryMetrics {
        TemplateNodeTextGeometryMetrics {
            horizontal_inset: 4.0,
            vertical_inset: 4.0,
            minimum_text_height: 13.0,
            edge_guard: 1.0,
        }
    }

    #[test]
    fn template_node_text_rect_uses_relative_shared_insets() {
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 30.0,
        };

        let text =
            text_rect_for_node_with_metrics(&TemplatePaneNodeData::default(), &rect, metrics());

        assert_eq!(
            text,
            FrameRect {
                x: 14.0,
                y: 24.0,
                width: 92.0,
                height: 22.0,
            }
        );
    }

    #[test]
    fn template_node_text_rect_preserves_caption_slot_in_short_rows() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 40.0,
            height: 14.0,
        };

        let text =
            text_rect_for_node_with_metrics(&TemplatePaneNodeData::default(), &rect, metrics());

        assert_eq!(text.y, 1.0);
        assert_eq!(text.height, 12.0);
        assert!(text.x >= rect.x);
        assert!(text.x + text.width <= rect.x + rect.width);
    }

    #[test]
    fn template_node_text_rect_rejects_non_finite_or_empty_parent_geometry() {
        let rect = FrameRect {
            x: f32::NAN,
            y: f32::INFINITY,
            width: -4.0,
            height: f32::NAN,
        };

        let text =
            text_rect_for_node_with_metrics(&TemplatePaneNodeData::default(), &rect, metrics());

        assert_eq!(text.x, 0.0);
        assert_eq!(text.y, 0.0);
        assert_eq!(text.width, 0.0);
        assert_eq!(text.height, 0.0);
    }
}

fn is_leading_icon_text_node(node: &TemplatePaneNodeData) -> bool {
    is_icon_node(node) && !is_icon_only_node(node) && !template_node_label(node, None).is_empty()
}
