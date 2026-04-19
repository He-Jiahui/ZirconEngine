use std::sync::Arc;

use serde_json::Value;

use crate::binding::UiEventBinding;

use super::super::{UiInvocationContext, UiInvocationError, UiRouteId};
use super::route_entry::RouteEntry;
use super::route_handler::RouteHandler;
use super::UiEventManager;

impl UiEventManager {
    pub fn register_route<F>(&mut self, binding: UiEventBinding, handler: F) -> UiRouteId
    where
        F: Fn(UiInvocationContext) -> Result<Value, UiInvocationError> + Send + Sync + 'static,
    {
        self.register_route_entry(binding, Some(Arc::new(handler)))
    }

    pub fn register_route_stub(&mut self, binding: UiEventBinding) -> UiRouteId {
        self.register_route_entry(binding, None)
    }

    pub fn route_binding(&self, route_id: UiRouteId) -> Option<UiEventBinding> {
        self.routes_by_id
            .get(&route_id)
            .map(|entry| entry.binding.clone())
    }

    pub fn route_id_for_binding(&self, binding: &UiEventBinding) -> Option<UiRouteId> {
        self.routes_by_binding
            .get(&binding.native_binding())
            .copied()
    }

    fn register_route_entry(
        &mut self,
        binding: UiEventBinding,
        handler: Option<RouteHandler>,
    ) -> UiRouteId {
        self.next_route_id += 1;
        let route_id = UiRouteId::new(self.next_route_id);
        let entry = RouteEntry {
            route_id,
            binding: binding.clone(),
            handler,
        };
        self.routes_by_binding
            .insert(binding.native_binding(), route_id);
        self.routes_by_id.insert(route_id, entry);
        route_id
    }
}
