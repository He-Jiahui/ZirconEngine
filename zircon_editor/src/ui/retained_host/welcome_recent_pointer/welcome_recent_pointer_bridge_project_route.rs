use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint};

use crate::ui::retained_host::welcome_recent_geometry::welcome_recent_row_geometry_with_metrics;

use super::helper::viewport_frame;
use super::welcome_recent_pointer_action::WelcomeRecentPointerAction;
use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;
use super::welcome_recent_pointer_route_intent::WelcomeRecentPointerRouteIntent;

impl WelcomeRecentPointerBridge {
    pub(super) fn project_route_at_point(
        &self,
        dispatched_route: Option<WelcomeRecentPointerRouteIntent>,
        point: UiPoint,
    ) -> Option<WelcomeRecentPointerRouteIntent> {
        match dispatched_route {
            Some(WelcomeRecentPointerRouteIntent::ListSurface) => self
                .item_route_at_point(point)
                .or(Some(WelcomeRecentPointerRouteIntent::ListSurface)),
            route => route,
        }
    }

    fn item_route_at_point(&self, point: UiPoint) -> Option<WelcomeRecentPointerRouteIntent> {
        let viewport = viewport_frame(&self.layout, self.layout_metrics);
        if self.layout.recent_project_paths.is_empty()
            || !point.x.is_finite()
            || !point.y.is_finite()
            || !frame_contains_interactive_point(viewport, point)
        {
            return None;
        }

        let first = welcome_recent_row_geometry_with_metrics(
            viewport,
            0,
            self.state.scroll_offset,
            self.layout_metrics,
        );
        let second = welcome_recent_row_geometry_with_metrics(
            viewport,
            1,
            self.state.scroll_offset,
            self.layout_metrics,
        );
        let row_pitch = second.row.y - first.row.y;
        let content_y = point.y - first.row.y;
        if !row_pitch.is_finite() || row_pitch <= 0.0 || !content_y.is_finite() || content_y < 0.0 {
            return None;
        }

        let item_index = (content_y / row_pitch).floor() as usize;
        let path = self.layout.recent_project_paths.get(item_index)?;
        let geometry = welcome_recent_row_geometry_with_metrics(
            viewport,
            item_index,
            self.state.scroll_offset,
            self.layout_metrics,
        );
        if !frame_contains_interactive_point(geometry.row, point) {
            return None;
        }
        let action = if frame_contains_interactive_point(geometry.remove, point) {
            Some(WelcomeRecentPointerAction::Remove)
        } else if frame_contains_interactive_point(geometry.open, point) {
            Some(WelcomeRecentPointerAction::Open)
        } else {
            None
        };

        Some(match action {
            Some(action) => WelcomeRecentPointerRouteIntent::Action {
                item_index,
                action,
                path: path.clone(),
            },
            None => WelcomeRecentPointerRouteIntent::Item(item_index),
        })
    }
}

fn frame_contains_interactive_point(frame: UiFrame, point: UiPoint) -> bool {
    frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
        && frame.contains_point(point)
}
