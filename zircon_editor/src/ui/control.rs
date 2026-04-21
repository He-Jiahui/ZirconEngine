use std::collections::BTreeMap;

use crossbeam_channel::Receiver;
use thiserror::Error;
use zircon_runtime::ui::{
    binding::UiEventBinding, event_ui::UiControlRequest, event_ui::UiControlResponse,
    event_ui::UiEventManager, event_ui::UiInvocationContext, event_ui::UiInvocationError,
    event_ui::UiNotification, event_ui::UiReflectionDiff, event_ui::UiReflectionSnapshot,
    event_ui::UiRouteId, event_ui::UiSubscriptionId,
};

use crate::ui::activity::{ActivityViewDescriptor, ActivityWindowDescriptor};

#[derive(Debug, Error)]
pub enum EditorUiError {
    #[error("activity view {0} already registered")]
    DuplicateActivityView(String),
    #[error("activity window {0} already registered")]
    DuplicateActivityWindow(String),
    #[error(transparent)]
    Invocation(#[from] UiInvocationError),
}

#[derive(Default)]
pub struct EditorUiControlService {
    activity_views: BTreeMap<String, ActivityViewDescriptor>,
    activity_windows: BTreeMap<String, ActivityWindowDescriptor>,
    event_manager: UiEventManager,
}

impl EditorUiControlService {
    pub fn register_activity_view(
        &mut self,
        descriptor: ActivityViewDescriptor,
    ) -> Result<(), EditorUiError> {
        if self.activity_views.contains_key(&descriptor.view_id) {
            return Err(EditorUiError::DuplicateActivityView(descriptor.view_id));
        }
        self.activity_views
            .insert(descriptor.view_id.clone(), descriptor);
        Ok(())
    }

    pub fn register_activity_window(
        &mut self,
        descriptor: ActivityWindowDescriptor,
    ) -> Result<(), EditorUiError> {
        if self.activity_windows.contains_key(&descriptor.window_id) {
            return Err(EditorUiError::DuplicateActivityWindow(descriptor.window_id));
        }
        self.activity_windows
            .insert(descriptor.window_id.clone(), descriptor);
        Ok(())
    }

    pub fn activity_view(&self, view_id: &str) -> Option<&ActivityViewDescriptor> {
        self.activity_views.get(view_id)
    }

    pub fn activity_window(&self, window_id: &str) -> Option<&ActivityWindowDescriptor> {
        self.activity_windows.get(window_id)
    }

    pub fn activity_views(&self) -> Vec<ActivityViewDescriptor> {
        self.activity_views.values().cloned().collect()
    }

    pub fn activity_windows(&self) -> Vec<ActivityWindowDescriptor> {
        self.activity_windows.values().cloned().collect()
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

    pub fn register_route_stub(&mut self, binding: UiEventBinding) -> UiRouteId {
        self.event_manager.register_route_stub(binding)
    }

    pub fn publish_snapshot(&mut self, snapshot: UiReflectionSnapshot) -> UiReflectionDiff {
        self.event_manager.replace_tree(snapshot)
    }

    pub fn query_tree(
        &self,
        tree_id: &zircon_runtime::ui::event_ui::UiTreeId,
    ) -> Option<UiReflectionSnapshot> {
        self.event_manager.query_tree(tree_id)
    }

    pub fn query_node(
        &self,
        node_path: &zircon_runtime::ui::event_ui::UiNodePath,
    ) -> Option<zircon_runtime::ui::event_ui::UiNodeDescriptor> {
        self.event_manager.query_node(node_path)
    }

    pub fn query_property(
        &self,
        node_path: &zircon_runtime::ui::event_ui::UiNodePath,
        property_name: &str,
    ) -> Option<zircon_runtime::ui::event_ui::UiPropertyDescriptor> {
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
