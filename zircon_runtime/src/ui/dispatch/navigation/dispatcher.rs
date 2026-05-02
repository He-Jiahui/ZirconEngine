use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

use crate::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime_interface::ui::dispatch::{
    UiNavigationDispatchContext, UiNavigationDispatchEffect, UiNavigationDispatchInvocation,
    UiNavigationDispatchResult,
};
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::surface::{UiNavigationEventKind, UiNavigationRoute};
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};

type NavigationHandler =
    Arc<dyn Fn(&UiNavigationDispatchContext) -> UiNavigationDispatchEffect + Send + Sync + 'static>;

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
