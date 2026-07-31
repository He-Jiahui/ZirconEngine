use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::ui::tree::UiRuntimeTreeRoutingExt;
use zircon_runtime_interface::ui::dispatch::{
    UiDispatchPhase, UiPointerDispatchContext, UiPointerDispatchEffect,
    UiPointerDispatchInvocation, UiPointerDispatchResult,
};
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::surface::{UiPointerEventKind, UiPointerRoute};
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

type PointerHandler =
    Arc<dyn Fn(&UiPointerDispatchContext) -> UiPointerDispatchEffect + Send + Sync + 'static>;

#[derive(Default)]
pub struct UiPointerDispatcher {
    handlers: BTreeMap<(UiNodeId, UiPointerEventKind), Vec<PointerHandler>>,
    phase_handlers: BTreeMap<(UiNodeId, UiPointerEventKind, UiDispatchPhase), Vec<PointerHandler>>,
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

    pub fn register_phase<F>(
        &mut self,
        node_id: UiNodeId,
        kind: UiPointerEventKind,
        phase: UiDispatchPhase,
        handler: F,
    ) where
        F: Fn(&UiPointerDispatchContext) -> UiPointerDispatchEffect + Send + Sync + 'static,
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
        let mut result = UiPointerDispatchResult::new(route.clone());

        if route_uses_preview_tunnel(&route) {
            for node_id in route.bubbled.iter().rev().copied() {
                if self.dispatch_phase_handlers(
                    &mut result,
                    node_id,
                    UiDispatchPhase::PreviewTunnel,
                    false,
                )? {
                    return Ok(result);
                }
            }
        }

        let mut visited = BTreeSet::new();
        let candidates = if let Some(captured) = route.captured {
            vec![captured]
        } else if let Some(target) = route.target {
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
                let phase = if node_id == candidate {
                    UiDispatchPhase::Target
                } else {
                    UiDispatchPhase::Bubble
                };
                if self.dispatch_phase_handlers(&mut result, node_id, phase, true)? {
                    return Ok(result);
                }
                if result.blocked_by == Some(node_id) {
                    continue 'candidate;
                }
            }
        }

        Ok(result)
    }

    fn dispatch_phase_handlers(
        &self,
        result: &mut UiPointerDispatchResult,
        node_id: UiNodeId,
        phase: UiDispatchPhase,
        include_unqualified_handlers: bool,
    ) -> Result<bool, UiTreeError> {
        let phase_handlers = self
            .phase_handlers
            .get(&(node_id, result.route.kind, phase))
            .into_iter()
            .flat_map(|handlers| handlers.iter());
        let unqualified_handlers = include_unqualified_handlers
            .then(|| self.handlers.get(&(node_id, result.route.kind)))
            .flatten()
            .into_iter()
            .flat_map(|handlers| handlers.iter());

        let context = UiPointerDispatchContext {
            node_id,
            phase,
            route: result.route.clone(),
        };
        for handler in phase_handlers.chain(unqualified_handlers) {
            let effect = handler(&context);
            if effect == UiPointerDispatchEffect::Unhandled {
                continue;
            }
            result.invocations.push(UiPointerDispatchInvocation {
                node_id,
                phase,
                effect,
            });
            if apply_pointer_effect(result, node_id, effect, phase) {
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
        && !route.bubbled.is_empty()
}

fn apply_pointer_effect(
    result: &mut UiPointerDispatchResult,
    node_id: UiNodeId,
    effect: UiPointerDispatchEffect,
    phase: UiDispatchPhase,
) -> bool {
    match effect {
        UiPointerDispatchEffect::Unhandled => false,
        UiPointerDispatchEffect::Handled => {
            result.handled_by = Some(node_id);
            true
        }
        UiPointerDispatchEffect::Blocked => {
            result.blocked_by = Some(node_id);
            phase == UiDispatchPhase::PreviewTunnel
        }
        UiPointerDispatchEffect::Passthrough => {
            result.passthrough.push(node_id);
            false
        }
        UiPointerDispatchEffect::CapturePointer => {
            result.captured_by = Some(node_id);
            result.handled_by = Some(node_id);
            true
        }
        UiPointerDispatchEffect::ReleasePointerCapture => {
            result.released_capture = Some(node_id);
            result.handled_by = Some(node_id);
            true
        }
        UiPointerDispatchEffect::SetFocus { .. } => {
            result.focus_changed_to = Some(node_id);
            result.handled_by = Some(node_id);
            true
        }
        UiPointerDispatchEffect::ClearFocus => {
            result.focus_cleared = true;
            result.handled_by = Some(node_id);
            true
        }
        UiPointerDispatchEffect::RequestDirty(flags) => {
            result.requested_dirty.layout |= flags.layout;
            result.requested_dirty.hit_test |= flags.hit_test;
            result.requested_dirty.render |= flags.render;
            result.requested_dirty.style |= flags.style;
            result.requested_dirty.text |= flags.text;
            result.requested_dirty.input |= flags.input;
            result.requested_dirty.visible_range |= flags.visible_range;
            false
        }
        UiPointerDispatchEffect::RequestDamage(frame) => {
            result.requested_damage.push(frame);
            false
        }
    }
}
