use std::collections::BTreeMap;
use std::sync::Arc;

use crate::ui::dispatch::visited_node_set::UiDispatchVisitedNodeSet;
use crate::ui::tree::UiRuntimeTreeRoutingExt;
use zircon_runtime_interface::ui::dispatch::{
    UiDispatchPhase, UiPointerDispatchContext, UiPointerDispatchEffect,
    UiPointerDispatchInvocation, UiPointerDispatchResult,
};
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiPointerEventKind, UiPointerRoute};
use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTree, UiTreeError};

type PointerHandler = Arc<
    dyn for<'route> Fn(&UiPointerDispatchContext<'route>) -> UiPointerDispatchEffect
        + Send
        + Sync
        + 'static,
>;

#[derive(Default)]
struct UiPointerDispatchState {
    invocations: Vec<UiPointerDispatchInvocation>,
    handled_by: Option<UiNodeId>,
    blocked_by: Option<UiNodeId>,
    passthrough: Vec<UiNodeId>,
    captured_by: Option<UiNodeId>,
    released_capture: Option<UiNodeId>,
    focus_changed_to: Option<UiNodeId>,
    focus_cleared: bool,
    requested_dirty: UiDirtyFlags,
    requested_damage: Vec<UiFrame>,
}

impl UiPointerDispatchState {
    fn finish(self, route: UiPointerRoute) -> UiPointerDispatchResult {
        let mut result = UiPointerDispatchResult::new(route);
        result.invocations = self.invocations;
        result.handled_by = self.handled_by;
        result.blocked_by = self.blocked_by;
        result.passthrough = self.passthrough;
        result.captured_by = self.captured_by;
        result.released_capture = self.released_capture;
        result.focus_changed_to = self.focus_changed_to;
        result.focus_cleared = self.focus_cleared;
        result.requested_dirty = self.requested_dirty;
        result.requested_damage = self.requested_damage;
        result
    }
}

#[derive(Default)]
pub struct UiPointerDispatcher {
    handlers: BTreeMap<(UiNodeId, UiPointerEventKind), Vec<PointerHandler>>,
    phase_handlers: BTreeMap<(UiNodeId, UiPointerEventKind, UiDispatchPhase), Vec<PointerHandler>>,
}

impl UiPointerDispatcher {
    pub fn register<F>(&mut self, node_id: UiNodeId, kind: UiPointerEventKind, handler: F)
    where
        F: for<'route> Fn(&UiPointerDispatchContext<'route>) -> UiPointerDispatchEffect
            + Send
            + Sync
            + 'static,
    {
        self.handlers
            .entry((node_id, kind))
            .or_default()
            .push(Arc::new(handler));
    }

    pub fn register_phase<F>(
        &mut self,
        node_id: UiNodeId,
        kind: UiPointerEventKind,
        phase: UiDispatchPhase,
        handler: F,
    ) where
        F: for<'route> Fn(&UiPointerDispatchContext<'route>) -> UiPointerDispatchEffect
            + Send
            + Sync
            + 'static,
    {
        self.phase_handlers
            .entry((node_id, kind, phase))
            .or_default()
            .push(Arc::new(handler));
    }

    pub fn dispatch(
        &self,
        tree: &UiTree,
        route: UiPointerRoute,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        self.dispatch_route(tree, route, false)
    }

    pub(crate) fn dispatch_surface_route(
        &self,
        tree: &UiTree,
        route: UiPointerRoute,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        self.dispatch_route(tree, route, true)
    }

    fn dispatch_route(
        &self,
        tree: &UiTree,
        route: UiPointerRoute,
        trust_cached_target_route: bool,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        let mut state = UiPointerDispatchState::default();
        if self.handlers.is_empty() && self.phase_handlers.is_empty() {
            return Ok(state.finish(route));
        }

        let mut propagation_stopped = false;
        if route_uses_preview_tunnel(&route) {
            for node_id in route.routing_root_to_leaf().iter().copied() {
                if self.dispatch_phase_handlers(
                    &route,
                    &mut state,
                    node_id,
                    UiDispatchPhase::PreviewTunnel,
                    false,
                )? {
                    propagation_stopped = true;
                    break;
                }
            }
        }
        if propagation_stopped {
            return Ok(state.finish(route));
        }

        let (injected_candidate, candidate_slice): (Option<UiNodeId>, &[UiNodeId]) =
            if let Some(captured) = route.captured {
                (Some(captured), &[])
            } else if let Some(target) = route.target {
                (
                    (!route.stacked.contains(&target)).then_some(target),
                    route.stacked.as_slice(),
                )
            } else {
                (None, route.root_targets.as_slice())
            };
        let expected_visited_len = route
            .routing_root_to_leaf()
            .len()
            .saturating_add(candidate_slice.len())
            .saturating_add(injected_candidate.is_some() as usize);
        let mut visited = UiDispatchVisitedNodeSet::with_expected_len(expected_visited_len);

        'candidate: for candidate in injected_candidate
            .into_iter()
            .chain(candidate_slice.iter().copied())
        {
            let stop_dispatch = if trust_cached_target_route
                && route.captured.is_none()
                && route.target == Some(candidate)
            {
                self.dispatch_candidate_route(
                    &route,
                    &mut state,
                    &mut visited,
                    candidate,
                    route.bubble_route(),
                )?
            } else {
                self.dispatch_candidate_route(
                    &route,
                    &mut state,
                    &mut visited,
                    candidate,
                    tree.bubble_route(candidate)?,
                )?
            };
            if stop_dispatch {
                break 'candidate;
            }
        }

        Ok(state.finish(route))
    }

    fn dispatch_candidate_route(
        &self,
        route: &UiPointerRoute,
        state: &mut UiPointerDispatchState,
        visited: &mut UiDispatchVisitedNodeSet,
        candidate: UiNodeId,
        bubble_route: impl IntoIterator<Item = UiNodeId>,
    ) -> Result<bool, UiTreeError> {
        for node_id in bubble_route {
            if !visited.insert(node_id) {
                continue;
            }
            let phase = if node_id == candidate {
                UiDispatchPhase::Target
            } else {
                UiDispatchPhase::Bubble
            };
            if self.dispatch_phase_handlers(route, state, node_id, phase, true)? {
                return Ok(true);
            }
            if state.blocked_by == Some(node_id) {
                return Ok(false);
            }
        }
        Ok(false)
    }

    fn dispatch_phase_handlers(
        &self,
        route: &UiPointerRoute,
        state: &mut UiPointerDispatchState,
        node_id: UiNodeId,
        phase: UiDispatchPhase,
        include_unqualified_handlers: bool,
    ) -> Result<bool, UiTreeError> {
        let phase_handlers = self.phase_handlers.get(&(node_id, route.kind, phase));
        let unqualified_handlers = include_unqualified_handlers
            .then(|| self.handlers.get(&(node_id, route.kind)))
            .flatten();

        if phase_handlers.is_none() && unqualified_handlers.is_none() {
            return Ok(false);
        }

        let context = UiPointerDispatchContext {
            node_id,
            phase,
            route,
        };
        let phase_handlers = phase_handlers
            .into_iter()
            .flat_map(|handlers| handlers.iter());
        let unqualified_handlers = unqualified_handlers
            .into_iter()
            .flat_map(|handlers| handlers.iter());
        for handler in phase_handlers.chain(unqualified_handlers) {
            let effect = handler(&context);
            if effect == UiPointerDispatchEffect::Unhandled {
                continue;
            }
            state.invocations.push(UiPointerDispatchInvocation {
                node_id,
                phase,
                effect,
            });
            if apply_pointer_effect(state, node_id, effect, phase) {
                return Ok(true);
            }
            if effect == UiPointerDispatchEffect::Blocked {
                return Ok(false);
            }
        }
        Ok(false)
    }
}

fn route_uses_preview_tunnel(route: &UiPointerRoute) -> bool {
    route.captured.is_none()
        && matches!(
            route.kind,
            UiPointerEventKind::Down | UiPointerEventKind::Up | UiPointerEventKind::Scroll
        )
        && !route.routing_root_to_leaf().is_empty()
}

fn apply_pointer_effect(
    state: &mut UiPointerDispatchState,
    node_id: UiNodeId,
    effect: UiPointerDispatchEffect,
    phase: UiDispatchPhase,
) -> bool {
    match effect {
        UiPointerDispatchEffect::Unhandled => false,
        UiPointerDispatchEffect::Handled => {
            state.handled_by = Some(node_id);
            true
        }
        UiPointerDispatchEffect::Blocked => {
            state.blocked_by = Some(node_id);
            phase == UiDispatchPhase::PreviewTunnel
        }
        UiPointerDispatchEffect::Passthrough => {
            state.passthrough.push(node_id);
            false
        }
        UiPointerDispatchEffect::CapturePointer => {
            state.captured_by = Some(node_id);
            state.handled_by = Some(node_id);
            true
        }
        UiPointerDispatchEffect::ReleasePointerCapture => {
            state.released_capture = Some(node_id);
            state.handled_by = Some(node_id);
            true
        }
        UiPointerDispatchEffect::SetFocus => {
            state.focus_changed_to = Some(node_id);
            state.handled_by = Some(node_id);
            true
        }
        UiPointerDispatchEffect::ClearFocus => {
            state.focus_cleared = true;
            state.handled_by = Some(node_id);
            true
        }
        UiPointerDispatchEffect::RequestDirty(flags) => {
            state.requested_dirty.layout |= flags.layout;
            state.requested_dirty.hit_test |= flags.hit_test;
            state.requested_dirty.render |= flags.render;
            state.requested_dirty.style |= flags.style;
            state.requested_dirty.text |= flags.text;
            state.requested_dirty.input |= flags.input;
            state.requested_dirty.visible_range |= flags.visible_range;
            false
        }
        UiPointerDispatchEffect::RequestDamage(frame) => {
            state.requested_damage.push(frame);
            false
        }
    }
}
