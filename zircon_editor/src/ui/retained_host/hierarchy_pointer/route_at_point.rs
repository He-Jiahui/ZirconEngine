use zircon_runtime_interface::ui::layout::UiPoint;

use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_route::HierarchyPointerRoute;
use super::row_metrics::hierarchy_row_width;
use super::viewport_frame::viewport_frame;

impl HierarchyPointerBridge {
    pub(super) fn route_at_point(&self, point: UiPoint) -> Option<HierarchyPointerRoute> {
        let viewport = viewport_frame(&self.layout);
        if !point.x.is_finite() || !point.y.is_finite() || !viewport.contains_point(point) {
            return None;
        }

        Some(
            self.node_route_at_point(point)
                .unwrap_or(HierarchyPointerRoute::ListSurface),
        )
    }

    fn node_route_at_point(&self, point: UiPoint) -> Option<HierarchyPointerRoute> {
        let viewport = viewport_frame(&self.layout);
        let row_width = hierarchy_row_width(self.layout.pane_width, self.row_metrics);
        let row_left = viewport.x + self.row_metrics.row_x;
        if point.x < row_left || point.x > row_left + row_width {
            return None;
        }

        let row_pitch = self.row_metrics.row_height + self.row_metrics.row_gap;
        if !row_pitch.is_finite() || row_pitch <= 0.0 {
            return None;
        }
        let content_y =
            point.y - viewport.y + self.state.scroll_offset.max(0.0) - self.row_metrics.row_y;
        if !content_y.is_finite() || content_y < 0.0 {
            return None;
        }
        let item_index = (content_y / row_pitch).floor() as usize;
        let offset_in_row = content_y - item_index as f32 * row_pitch;
        if offset_in_row >= self.row_metrics.row_height || item_index >= self.layout.item_count {
            return None;
        }
        Some(HierarchyPointerRoute::Node { item_index })
    }
}
