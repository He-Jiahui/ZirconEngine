use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::ui::tree::UiRuntimeTreeRoutingExt;
use zircon_runtime_interface::ui::dispatch::{
    UiPointerDispatchContext, UiPointerDispatchEffect, UiPointerDispatchInvocation,
    UiPointerDispatchResult,
};
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::surface::{UiPointerEventKind, UiPointerRoute};
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

type PointerHandler =
    Arc<dyn Fn(&UiPointerDispatchContext) -> UiPointerDispatchEffect + Send + Sync + 'static>;

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
