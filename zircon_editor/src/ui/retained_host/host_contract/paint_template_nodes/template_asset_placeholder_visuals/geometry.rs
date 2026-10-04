//! 资产缩略图井从实际节点框与宿主密度取得内框，类型化预览与普通占位使用不同留白。
//! fallback 在画井和请求预览前使用这一有限尺寸门槛，退化框不再产生命令。

use super::{
    is_typed_thumbnail_visual, FrameRect, TemplatePaneNodeData, WorkbenchAssetVisualMetrics,
    TYPED_THUMBNAIL_SURFACE_INSET_RATIO, VISUAL_SURFACE_INSET_RATIO,
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
#[path = "tests/geometry.rs"]
mod tests;
