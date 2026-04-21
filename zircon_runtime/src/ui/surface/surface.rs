use serde::{Deserialize, Serialize};

use crate::ui::dispatch::{
    UiNavigationDispatchResult, UiNavigationDispatcher, UiPointerDispatchResult,
    UiPointerDispatcher, UiPointerEvent,
};
use crate::ui::event_ui::{UiNodeId, UiTreeId};
use crate::ui::layout::{compute_layout_tree, UiPoint, UiSize};
use crate::ui::tree::{UiHitTestIndex, UiHitTestResult, UiTree, UiTreeError};

use super::{
    UiFocusState, UiNavigationEventKind, UiNavigationRoute, UiNavigationState, UiPointerButton,
    UiPointerEventKind, UiPointerRoute, UiRenderExtract, UiRenderList,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurface {
    pub tree: UiTree,
    pub hit_test: UiHitTestIndex,
    pub focus: UiFocusState,
    pub navigation: UiNavigationState,
    pub render_extract: UiRenderExtract,
}

impl UiSurface {
    pub fn new(tree_id: UiTreeId) -> Self {
        Self {
            tree: UiTree::new(tree_id.clone()),
            hit_test: UiHitTestIndex::default(),
            focus: UiFocusState::default(),
            navigation: UiNavigationState::default(),
            render_extract: UiRenderExtract {
                tree_id,
                list: UiRenderList::default(),
            },
        }
    }

    pub fn rebuild(&mut self) {
        self.hit_test.rebuild(&self.tree);
        self.render_extract = UiRenderExtract::from_tree(&self.tree);
    }

    pub fn compute_layout(&mut self, root_size: UiSize) -> Result<(), UiTreeError> {
        compute_layout_tree(&mut self.tree, root_size)?;
        self.rebuild();
        Ok(())
    }

    pub fn hit_test(&self, point: UiPoint) -> UiHitTestResult {
        self.hit_test.hit_test(&self.tree, point)
    }

    pub fn bubble_route(&self, node_id: UiNodeId) -> Result<Vec<UiNodeId>, UiTreeError> {
        self.tree.bubble_route(node_id)
    }

    pub fn focused_route(&self) -> Vec<UiNodeId> {
        self.focus
            .focused
            .and_then(|node_id| self.tree.bubble_route(node_id).ok())
            .unwrap_or_default()
    }

    pub fn focus_node(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        if !(node.state_flags.visible && node.state_flags.enabled && node.state_flags.focusable) {
            return Err(UiTreeError::MissingNode(node_id));
        }
        self.focus.focused = Some(node_id);
        self.navigation.navigation_root = Some(node_id);
        self.navigation.focus_visible = true;
        Ok(())
    }

    pub fn clear_focus(&mut self) {
        self.focus.focused = None;
        self.navigation.navigation_root = None;
        self.navigation.focus_visible = false;
    }

    pub fn capture_pointer(&mut self, node_id: UiNodeId) -> Result<(), UiTreeError> {
        self.tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        self.focus.captured = Some(node_id);
        Ok(())
    }

    pub fn release_pointer_capture(&mut self) -> Option<UiNodeId> {
        self.focus.captured.take()
    }

    pub fn route_pointer_event(
        &mut self,
        kind: UiPointerEventKind,
        point: UiPoint,
    ) -> Result<UiPointerRoute, UiTreeError> {
        self.route_pointer_event_with_details(kind, point, None, 0.0)
    }

    pub fn dispatch_pointer_event(
        &mut self,
        dispatcher: &UiPointerDispatcher,
        event: UiPointerEvent,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        let route = self.route_pointer_event_with_details(
            event.kind,
            event.point,
            event.button,
            event.scroll_delta,
        )?;
        let mut result = dispatcher.dispatch(&self.tree, route.clone())?;
        if let Some(node_id) = result.captured_by {
            self.focus.captured = Some(node_id);
        } else if matches!(event.kind, UiPointerEventKind::Scroll)
            && result.handled_by.is_none()
            && result.blocked_by.is_none()
        {
            let candidates = if !route.stacked.is_empty() {
                route.stacked.as_slice()
            } else {
                route.root_targets.as_slice()
            };
            if let Some(node_id) = self.tree.first_scrollable_in_candidates(candidates)? {
                let _ = self.tree.scroll_by(node_id, event.scroll_delta)?;
                result.handled_by = Some(node_id);
            }
        }
        Ok(result)
    }

    fn route_pointer_event_with_details(
        &mut self,
        kind: UiPointerEventKind,
        point: UiPoint,
        button: Option<UiPointerButton>,
        scroll_delta: f32,
    ) -> Result<UiPointerRoute, UiTreeError> {
        let hit = self.hit_test(point);
        let previous_hovered = self.focus.hovered.clone();
        let captured = self.focus.captured;
        let target = captured.or(hit.top_hit);
        let bubbled = match target {
            Some(node_id) => self.tree.bubble_route(node_id)?,
            None => Vec::new(),
        };

        self.focus.hovered = hit.stacked.clone();
        if matches!(kind, UiPointerEventKind::Down) {
            if let Some(focus_target) = self.tree.first_focusable_in_route(&bubbled)? {
                self.focus_node(focus_target)?;
            }
        }
        if matches!(kind, UiPointerEventKind::Up) {
            self.focus.captured = None;
        }

        Ok(UiPointerRoute {
            kind,
            button,
            point,
            scroll_delta,
            target,
            bubbled,
            stacked: hit.stacked.clone(),
            entered: diff_nodes(&hit.stacked, &previous_hovered),
            left: diff_nodes(&previous_hovered, &hit.stacked),
            captured,
            focused: self.focus.focused,
            fallback_to_root: target.is_none(),
            root_targets: if target.is_none() {
                self.tree.roots.clone()
            } else {
                Vec::new()
            },
        })
    }

    pub fn route_navigation_event(
        &self,
        kind: UiNavigationEventKind,
    ) -> Result<UiNavigationRoute, UiTreeError> {
        let target = self.focus.focused.or(self.navigation.navigation_root);
        let bubbled = match target {
            Some(node_id) => self.tree.bubble_route(node_id)?,
            None => Vec::new(),
        };
        Ok(UiNavigationRoute {
            kind,
            target,
            bubbled,
            fallback_to_root: target.is_none(),
            root_targets: if target.is_none() {
                self.tree.roots.clone()
            } else {
                Vec::new()
            },
        })
    }

    pub fn dispatch_navigation_event(
        &mut self,
        dispatcher: &UiNavigationDispatcher,
        kind: UiNavigationEventKind,
    ) -> Result<UiNavigationDispatchResult, UiTreeError> {
        let route = self.route_navigation_event(kind)?;
        let mut result = dispatcher.dispatch(&self.tree, route.clone())?;
        if result.focus_changed_to.is_none() {
            if let Some(node_id) = self.tree.next_focusable_target(route.target, route.kind)? {
                result.handled_by = Some(route.target.unwrap_or(node_id));
                result.focus_changed_to = Some(node_id);
            }
        }
        if let Some(node_id) = result.focus_changed_to {
            self.focus_node(node_id)?;
        }
        Ok(result)
    }
}

fn diff_nodes(current: &[UiNodeId], previous: &[UiNodeId]) -> Vec<UiNodeId> {
    current
        .iter()
        .filter(|node_id| !previous.contains(node_id))
        .copied()
        .collect()
}
