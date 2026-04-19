use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::surface::{
    UiNavigationEventKind, UiNavigationRoute, UiPointerButton, UiPointerEventKind, UiPointerRoute,
};
use crate::tree::UiTreeError;
use crate::event_ui::UiNodeId;
use crate::UiTree;

type PointerHandler =
    Arc<dyn Fn(&UiPointerDispatchContext) -> UiPointerDispatchEffect + Send + Sync + 'static>;
type NavigationHandler =
    Arc<dyn Fn(&UiNavigationDispatchContext) -> UiNavigationDispatchEffect + Send + Sync + 'static>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiPointerDispatchEffect {
    #[default]
    Unhandled,
    Handled,
    Blocked,
    Passthrough,
    Captured,
}

impl UiPointerDispatchEffect {
    pub const fn handled() -> Self {
        Self::Handled
    }

    pub const fn blocked() -> Self {
        Self::Blocked
    }

    pub const fn passthrough() -> Self {
        Self::Passthrough
    }

    pub const fn capture() -> Self {
        Self::Captured
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerEvent {
    pub kind: UiPointerEventKind,
    pub button: Option<UiPointerButton>,
    pub point: crate::UiPoint,
    pub scroll_delta: f32,
}

impl UiPointerEvent {
    pub const fn new(kind: UiPointerEventKind, point: crate::UiPoint) -> Self {
        Self {
            kind,
            button: None,
            point,
            scroll_delta: 0.0,
        }
    }

    pub const fn with_button(mut self, button: UiPointerButton) -> Self {
        self.button = Some(button);
        self
    }

    pub const fn with_scroll_delta(mut self, scroll_delta: f32) -> Self {
        self.scroll_delta = scroll_delta;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerDispatchContext {
    pub node_id: UiNodeId,
    pub route: UiPointerRoute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPointerDispatchInvocation {
    pub node_id: UiNodeId,
    pub effect: UiPointerDispatchEffect,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerDispatchResult {
    pub route: UiPointerRoute,
    pub invocations: Vec<UiPointerDispatchInvocation>,
    pub handled_by: Option<UiNodeId>,
    pub blocked_by: Option<UiNodeId>,
    pub passthrough: Vec<UiNodeId>,
    pub captured_by: Option<UiNodeId>,
}

impl UiPointerDispatchResult {
    pub fn new(route: UiPointerRoute) -> Self {
        Self {
            route,
            invocations: Vec::new(),
            handled_by: None,
            blocked_by: None,
            passthrough: Vec::new(),
            captured_by: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiNavigationDispatchEffect {
    Unhandled,
    Handled,
    Focus(UiNodeId),
}

impl UiNavigationDispatchEffect {
    pub const fn handled() -> Self {
        Self::Handled
    }

    pub const fn focus(node_id: UiNodeId) -> Self {
        Self::Focus(node_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationDispatchContext {
    pub node_id: UiNodeId,
    pub route: UiNavigationRoute,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationDispatchInvocation {
    pub node_id: UiNodeId,
    pub effect: UiNavigationDispatchEffect,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationDispatchResult {
    pub route: UiNavigationRoute,
    pub invocations: Vec<UiNavigationDispatchInvocation>,
    pub handled_by: Option<UiNodeId>,
    pub focus_changed_to: Option<UiNodeId>,
}

impl UiNavigationDispatchResult {
    pub fn new(route: UiNavigationRoute) -> Self {
        Self {
            route,
            invocations: Vec::new(),
            handled_by: None,
            focus_changed_to: None,
        }
    }
}

#[derive(Default)]
pub struct UiPointerDispatcher {
    handlers: BTreeMap<(UiNodeId, UiPointerEventKind), Vec<PointerHandler>>,
}

impl UiPointerDispatcher {
    pub fn register<F>(&mut self, node_id: UiNodeId, kind: UiPointerEventKind, handler: F)
    where
        F: Fn(&UiPointerDispatchContext) -> UiPointerDispatchEffect + Send + Sync + 'static,
    {
        self.handlers
            .entry((node_id, kind))
            .or_default()
            .push(Arc::new(handler));
    }

    pub fn dispatch(
        &self,
        tree: &UiTree,
        route: UiPointerRoute,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        let mut result = UiPointerDispatchResult::new(route.clone());
        let mut visited = BTreeSet::new();
        let candidates = if let Some(target) = route.target {
            let mut candidates = route.stacked.clone();
            if !candidates.contains(&target) {
                candidates.insert(0, target);
            }
            candidates
        } else {
            route.root_targets.clone()
        };

        'candidate: for candidate in candidates {
            let bubble = tree.bubble_route(candidate)?;
            for node_id in bubble {
                if !visited.insert(node_id) {
                    continue;
                }
                let Some(handlers) = self.handlers.get(&(node_id, route.kind)) else {
                    continue;
                };
                let context = UiPointerDispatchContext {
                    node_id,
                    route: route.clone(),
                };
                for handler in handlers {
                    let effect = handler(&context);
                    if effect == UiPointerDispatchEffect::Unhandled {
                        continue;
                    }
                    result
                        .invocations
                        .push(UiPointerDispatchInvocation { node_id, effect });
                    match effect {
                        UiPointerDispatchEffect::Unhandled => {}
                        UiPointerDispatchEffect::Handled => {
                            result.handled_by = Some(node_id);
                            return Ok(result);
                        }
                        UiPointerDispatchEffect::Blocked => {
                            result.blocked_by = Some(node_id);
                            continue 'candidate;
                        }
                        UiPointerDispatchEffect::Passthrough => {
                            result.passthrough.push(node_id);
                        }
                        UiPointerDispatchEffect::Captured => {
                            result.captured_by = Some(node_id);
                            result.handled_by = Some(node_id);
                            return Ok(result);
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}

#[derive(Default)]
pub struct UiNavigationDispatcher {
    handlers: BTreeMap<(UiNodeId, UiNavigationEventKind), Vec<NavigationHandler>>,
}

impl UiNavigationDispatcher {
    pub fn register<F>(&mut self, node_id: UiNodeId, kind: UiNavigationEventKind, handler: F)
    where
        F: Fn(&UiNavigationDispatchContext) -> UiNavigationDispatchEffect + Send + Sync + 'static,
    {
        self.handlers
            .entry((node_id, kind))
            .or_default()
            .push(Arc::new(handler));
    }

    pub fn dispatch(
        &self,
        tree: &UiTree,
        route: UiNavigationRoute,
    ) -> Result<UiNavigationDispatchResult, UiTreeError> {
        let mut result = UiNavigationDispatchResult::new(route.clone());
        let mut visited = BTreeSet::new();
        let candidates = if route.target.is_some() {
            route.bubbled.clone()
        } else {
            route.root_targets.clone()
        };

        for node_id in candidates {
            if !visited.insert(node_id) {
                continue;
            }
            let Some(handlers) = self.handlers.get(&(node_id, route.kind)) else {
                continue;
            };
            let context = UiNavigationDispatchContext {
                node_id,
                route: route.clone(),
            };
            for handler in handlers {
                let effect = handler(&context);
                if effect == UiNavigationDispatchEffect::Unhandled {
                    continue;
                }
                result.invocations.push(UiNavigationDispatchInvocation {
                    node_id,
                    effect: effect.clone(),
                });
                match effect {
                    UiNavigationDispatchEffect::Unhandled => {}
                    UiNavigationDispatchEffect::Handled => {
                        result.handled_by = Some(node_id);
                        return Ok(result);
                    }
                    UiNavigationDispatchEffect::Focus(target_node_id) => {
                        tree.node(target_node_id)
                            .ok_or(UiTreeError::MissingNode(target_node_id))?;
                        result.handled_by = Some(node_id);
                        result.focus_changed_to = Some(target_node_id);
                        return Ok(result);
                    }
                }
            }
        }

        Ok(result)
    }
}
