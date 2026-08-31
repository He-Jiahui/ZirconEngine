use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint};

use crate::ui::retained_host::welcome_recent_geometry::welcome_recent_row_geometry_with_metrics;

use super::helper::viewport_frame;
use super::welcome_recent_pointer_action::WelcomeRecentPointerAction;
use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;
use super::welcome_recent_pointer_route::WelcomeRecentPointerRoute;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WelcomeRecentPointerHit {
    Item(usize),
    Action {
        item_index: usize,
        action: WelcomeRecentPointerAction,
    },
    ListSurface,
}

impl WelcomeRecentPointerBridge {
    pub(super) fn route_at_point(&self, point: UiPoint) -> Option<WelcomeRecentPointerHit> {
        if !self.viewport_contains(point) {
            return None;
        }

        self.item_hit_at_point(point)
            .or(Some(WelcomeRecentPointerHit::ListSurface))
    }

    pub(super) fn viewport_contains(&self, point: UiPoint) -> bool {
        let viewport = viewport_frame(&self.layout);
        point.x.is_finite()
            && point.y.is_finite()
            && frame_contains_interactive_point(viewport, point)
    }

    fn item_hit_at_point(&self, point: UiPoint) -> Option<WelcomeRecentPointerHit> {
        if self.layout.recent_project_paths.is_empty() {
            return None;
        }

        let viewport = viewport_frame(&self.layout);

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
        if item_index >= self.layout.recent_project_paths.len() {
            return None;
        }
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
        } else if frame_contains_interactive_point(geometry.recover, point) {
            Some(WelcomeRecentPointerAction::Recover)
        } else if frame_contains_interactive_point(geometry.safe, point) {
            Some(WelcomeRecentPointerAction::Safe)
        } else if frame_contains_interactive_point(geometry.open, point) {
            Some(WelcomeRecentPointerAction::Open)
        } else {
            None
        };

        Some(match action {
            Some(action) => WelcomeRecentPointerHit::Action { item_index, action },
            None => WelcomeRecentPointerHit::Item(item_index),
        })
    }

    pub(super) fn apply_hit_state(&mut self, hit: Option<WelcomeRecentPointerHit>) {
        match hit {
            Some(WelcomeRecentPointerHit::Action { item_index, action }) => {
                self.state.hovered_item_index = Some(item_index);
                self.state.hovered_action = Some(action);
            }
            Some(WelcomeRecentPointerHit::Item(item_index)) => {
                self.state.hovered_item_index = Some(item_index);
                self.state.hovered_action = None;
            }
            Some(WelcomeRecentPointerHit::ListSurface) | None => {
                self.state.hovered_item_index = None;
                self.state.hovered_action = None;
            }
        }
    }

    pub(crate) fn action_target_for_route(
        &self,
        route: WelcomeRecentPointerRoute,
    ) -> Option<(WelcomeRecentPointerAction, &str)> {
        let WelcomeRecentPointerRoute::Action { item_index, action } = route else {
            return None;
        };
        self.layout
            .recent_project_paths
            .get(item_index)
            .map(|path| (action, path.as_str()))
    }
}

impl WelcomeRecentPointerHit {
    pub(super) const fn to_public_route(self) -> WelcomeRecentPointerRoute {
        match self {
            Self::Action { item_index, action } => {
                WelcomeRecentPointerRoute::Action { item_index, action }
            }
            Self::Item(_) | Self::ListSurface => WelcomeRecentPointerRoute::ListSurface,
        }
    }
}

fn frame_contains_interactive_point(frame: UiFrame, point: UiPoint) -> bool {
    frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
        && frame.contains_point(point)
}
