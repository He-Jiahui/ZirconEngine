use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crossbeam_channel::Receiver;
use zircon_runtime::ui::event_ui::UiEventManager;
use zircon_runtime_interface::ui::{
    binding::UiEventBinding,
    event_ui::{
        UiControlRequest, UiControlResponse, UiInvocationContext, UiInvocationError,
        UiNodeDescriptor, UiNodePath, UiNotification, UiPropertyDescriptor, UiReflectionDiff,
        UiReflectionNodePatch, UiReflectionSnapshot, UiRouteId, UiSubscriptionId, UiTreeId,
    },
};

use super::error::EditorUiError;
use crate::ui::activity::{ActivityViewDescriptor, ActivityWindowDescriptor};

#[cfg(test)]
#[path = "service/activity_registry_hash_tests.rs"]
mod activity_registry_hash_tests;

#[derive(Default)]
pub struct EditorUiControlService {
    activity_views: HashMap<String, ActivityViewDescriptor>,
    activity_windows: HashMap<String, ActivityWindowDescriptor>,
    event_manager: UiEventManager,
}

impl EditorUiControlService {
    pub fn register_activity_view(
        &mut self,
        descriptor: ActivityViewDescriptor,
    ) -> Result<(), EditorUiError> {
        match self.activity_views.entry(descriptor.view_id.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(descriptor);
                Ok(())
            }
            Entry::Occupied(_) => Err(EditorUiError::DuplicateActivityView(descriptor.view_id)),
        }
    }

    pub fn register_activity_window(
        &mut self,
        descriptor: ActivityWindowDescriptor,
    ) -> Result<(), EditorUiError> {
        match self.activity_windows.entry(descriptor.window_id.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(descriptor);
                Ok(())
            }
            Entry::Occupied(_) => Err(EditorUiError::DuplicateActivityWindow(descriptor.window_id)),
        }
    }

    pub fn activity_view(&self, view_id: &str) -> Option<&ActivityViewDescriptor> {
        self.activity_views.get(view_id)
    }

    pub fn activity_window(&self, window_id: &str) -> Option<&ActivityWindowDescriptor> {
        self.activity_windows.get(window_id)
    }

    pub fn activity_views(&self) -> Vec<ActivityViewDescriptor> {
        let mut views = self.activity_views.values().cloned().collect::<Vec<_>>();
        views.sort_by(|left, right| left.view_id.cmp(&right.view_id));
        views
    }

    pub fn activity_windows(&self) -> Vec<ActivityWindowDescriptor> {
        let mut windows = self.activity_windows.values().cloned().collect::<Vec<_>>();
        windows.sort_by(|left, right| left.window_id.cmp(&right.window_id));
        windows
    }

    pub fn register_route<F>(&mut self, binding: UiEventBinding, handler: F) -> UiRouteId
    where
        F: Fn(UiInvocationContext) -> Result<serde_json::Value, UiInvocationError>
            + Send
            + Sync
            + 'static,
    {
        self.event_manager.register_route(binding, handler)
    }

    pub fn register_binding_route(&mut self, binding: UiEventBinding) -> UiRouteId {
        self.event_manager.register_binding_route(binding)
    }

    pub fn publish_snapshot(&mut self, snapshot: UiReflectionSnapshot) -> UiReflectionDiff {
        self.event_manager.replace_tree(snapshot)
    }

    pub fn apply_reflection_patches(
        &mut self,
        patches: &[UiReflectionNodePatch],
    ) -> Result<Vec<UiReflectionDiff>, UiInvocationError> {
        self.event_manager.apply_reflection_patches(patches)
    }

    pub fn query_tree(&self, tree_id: &UiTreeId) -> Option<UiReflectionSnapshot> {
        self.event_manager.query_tree(tree_id)
    }

    pub fn query_node(&self, node_path: &UiNodePath) -> Option<UiNodeDescriptor> {
        self.event_manager.query_node(node_path)
    }

    pub fn query_property(
        &self,
        node_path: &UiNodePath,
        property_name: &str,
    ) -> Option<UiPropertyDescriptor> {
        self.event_manager.query_property(node_path, property_name)
    }

    pub fn route_binding(&self, route_id: UiRouteId) -> Option<UiEventBinding> {
        self.event_manager.route_binding(route_id)
    }

    pub fn route_id_for_binding(&self, binding: &UiEventBinding) -> Option<UiRouteId> {
        self.event_manager.route_id_for_binding(binding)
    }

    pub fn handle_request(&mut self, request: UiControlRequest) -> UiControlResponse {
        self.event_manager.handle_request(request)
    }

    pub fn subscribe(&mut self) -> (UiSubscriptionId, Receiver<UiNotification>) {
        self.event_manager.subscribe()
    }
}
