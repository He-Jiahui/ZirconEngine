use super::{
    FrameRect, TYPED_THUMBNAIL_SURFACE_INSET_RATIO, TemplatePaneNodeData,
    VISUAL_SURFACE_INSET_RATIO, WorkbenchAssetVisualMetrics, is_typed_thumbnail_visual,
};

pub(super) fn thumbnail_surface_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    metrics: WorkbenchAssetVisualMetrics,
) -> Option<FrameRect> {
    if !has_paintable_thumbnail_extent(rect) {
        return None;
    }
    let shortest_edge = rect.width.min(rect.height);
    let min_inset = if is_typed_thumbnail_visual(node) {
        metrics.typed_surface_min_inset
    } else {
        metrics.visual_surface_min_inset
    };
    if shortest_edge <= min_inset * 2.0 {
        return None;
    }
    let inset = thumbnail_surface_inset(node, shortest_edge, metrics);
    let width = (rect.width - inset * 2.0).max(0.0);
    let height = (rect.height - inset * 2.0).max(0.0);
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return None;
    }
    let inner = FrameRect {
        x: rect.x + inset,
        y: rect.y + inset,
        width,
        height,
    };
    has_paintable_thumbnail_extent(&inner).then_some(inner)
}

pub(super) fn has_paintable_thumbnail_extent(rect: &FrameRect) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
        && (rect.x + rect.width).is_finite()
        && (rect.y + rect.height).is_finite()
}

pub(super) fn frame_is_within(outer: &FrameRect, inner: &FrameRect) -> bool {
    has_paintable_thumbnail_extent(outer)
        && has_paintable_thumbnail_extent(inner)
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

fn thumbnail_surface_inset(
    node: &TemplatePaneNodeData,
    shortest_edge: f32,
    metrics: WorkbenchAssetVisualMetrics,
) -> f32 {
    if is_typed_thumbnail_visual(node) {
        return (shortest_edge * TYPED_THUMBNAIL_SURFACE_INSET_RATIO).clamp(
            metrics.typed_surface_min_inset,
            metrics.typed_surface_max_inset,
        );
    }
    (shortest_edge * VISUAL_SURFACE_INSET_RATIO).clamp(
        metrics.visual_surface_min_inset,
        metrics.visual_surface_max_inset,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        FrameRect, TemplatePaneNodeData, has_paintable_thumbnail_extent, thumbnail_surface_rect,
    };
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn thumbnail_geometry_rejects_collapsed_non_finite_and_overflowed_frames() {
        let node = TemplatePaneNodeData::default();
        let metrics = super::super::asset_visual_metrics_from_host(METRICS);
        let valid = FrameRect {
            x: 12.0,
            y: 8.0,
            width: 74.0,
            height: 42.0,
        };

        assert!(has_paintable_thumbnail_extent(&valid));
        assert!(thumbnail_surface_rect(&node, &valid, metrics).is_some());
        assert!(!has_paintable_thumbnail_extent(&FrameRect {
            width: 0.0,
            ..valid.clone()
        }));
        assert!(
            thumbnail_surface_rect(
                &node,
                &FrameRect {
                    x: f32::NAN,
                    ..valid.clone()
                },
                metrics,
            )
            .is_none()
        );
        assert!(!has_paintable_thumbnail_extent(&FrameRect {
            x: f32::MAX,
            ..valid
        }));
    }
}
