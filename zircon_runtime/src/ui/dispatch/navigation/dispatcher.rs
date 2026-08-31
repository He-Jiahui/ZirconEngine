use std::collections::BTreeMap;
use std::sync::Arc;

use crate::ui::dispatch::visited_node_set::UiDispatchVisitedNodeSet;
use zircon_runtime_interface::ui::dispatch::{
    UiNavigationDispatchContext, UiNavigationDispatchEffect, UiNavigationDispatchInvocation,
    UiNavigationDispatchResult,
};
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::surface::{UiNavigationEventKind, UiNavigationRoute};
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

type NavigationHandler = Arc<
    dyn for<'route> Fn(&UiNavigationDispatchContext<'route>) -> UiNavigationDispatchEffect
        + Send
        + Sync
        + 'static,
>;

#[derive(Default)]
pub struct UiNavigationDispatcher {
    handlers: BTreeMap<(UiNodeId, UiNavigationEventKind), Vec<NavigationHandler>>,
}

impl UiNavigationDispatcher {
    pub fn register<F>(&mut self, node_id: UiNodeId, kind: UiNavigationEventKind, handler: F)
    where
        F: for<'route> Fn(&UiNavigationDispatchContext<'route>) -> UiNavigationDispatchEffect
            + Send
            + Sync
            + 'static,
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
        let route_kind = route.kind;
        let candidates: &[UiNodeId] = if route.target.is_some() {
            &route.bubbled
        } else {
            &route.root_targets
        };
        if candidates.is_empty() || self.handlers.is_empty() {
            return Ok(UiNavigationDispatchResult::new(route));
        }
        let mut visited = UiDispatchVisitedNodeSet::with_expected_len(candidates.len());
        let mut invocations = Vec::new();
        let mut handled_by = None;
        let mut focus_changed_to = None;

        'route: for node_id in candidates.iter().copied() {
            if !visited.insert(node_id) {
                continue;
            }
            let Some(handlers) = self.handlers.get(&(node_id, route_kind)) else {
                continue;
            };
            let context = UiNavigationDispatchContext {
                node_id,
                route: &route,
            };
            for handler in handlers {
                let effect = handler(&context);
                match effect {
                    UiNavigationDispatchEffect::Unhandled => continue,
                    UiNavigationDispatchEffect::Handled => {
                        invocations.push(UiNavigationDispatchInvocation {
                            node_id,
                            effect: UiNavigationDispatchEffect::Handled,
                        });
                        handled_by = Some(node_id);
                        break 'route;
                    }
                    UiNavigationDispatchEffect::Focus(target_node_id) => {
                        tree.node(target_node_id)
                            .ok_or(UiTreeError::MissingNode(target_node_id))?;
                        invocations.push(UiNavigationDispatchInvocation {
                            node_id,
                            effect: UiNavigationDispatchEffect::Focus(target_node_id),
                        });
                        handled_by = Some(node_id);
                        focus_changed_to = Some(target_node_id);
                        break 'route;
                    }
                }
            }
        }

        let mut result = UiNavigationDispatchResult::new(route);
        result.invocations = invocations;
        result.handled_by = handled_by;
        result.focus_changed_to = focus_changed_to;
        Ok(result)
    }
}
