use std::collections::BTreeMap;
use std::sync::Arc;

use crossbeam_channel::{unbounded, Receiver, Sender};
use serde_json::Value;

use crate::{UiBindingValue, UiEventBinding};

use super::{
    UiControlRequest, UiControlResponse, UiInvocationContext, UiInvocationError,
    UiInvocationResult, UiInvocationSource, UiNodeDescriptor, UiNodeId, UiNodePath, UiNotification,
    UiPropertyDescriptor, UiReflectionDiff, UiReflectionSnapshot, UiRouteId, UiSubscriptionId,
    UiTreeId,
};

type RouteHandler =
    Arc<dyn Fn(UiInvocationContext) -> Result<Value, UiInvocationError> + Send + Sync + 'static>;

#[derive(Clone)]
struct RouteEntry {
    route_id: UiRouteId,
    binding: UiEventBinding,
    handler: Option<RouteHandler>,
}

#[derive(Default)]
pub struct UiEventManager {
    next_route_id: u64,
    next_subscription_id: u64,
    routes_by_id: BTreeMap<UiRouteId, RouteEntry>,
    routes_by_binding: BTreeMap<String, UiRouteId>,
    trees: BTreeMap<UiTreeId, UiReflectionSnapshot>,
    node_index: BTreeMap<UiNodePath, (UiTreeId, UiNodeId)>,
    subscriptions: BTreeMap<UiSubscriptionId, Sender<UiNotification>>,
}

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
        self.routes_by_id.get(&route_id).map(|entry| entry.binding.clone())
    }

    pub fn route_id_for_binding(&self, binding: &UiEventBinding) -> Option<UiRouteId> {
        self.routes_by_binding.get(&binding.native_binding()).copied()
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

    pub fn subscribe(&mut self) -> (UiSubscriptionId, Receiver<UiNotification>) {
        self.next_subscription_id += 1;
        let subscription_id = UiSubscriptionId::new(self.next_subscription_id);
        let (tx, rx) = unbounded();
        self.subscriptions.insert(subscription_id, tx);
        (subscription_id, rx)
    }

    pub fn unsubscribe(&mut self, subscription_id: UiSubscriptionId) -> bool {
        self.subscriptions.remove(&subscription_id).is_some()
    }

    pub fn invoke_route(
        &self,
        route_id: UiRouteId,
        arguments: Vec<UiBindingValue>,
    ) -> UiInvocationResult {
        self.invoke_route_internal(route_id, arguments, UiInvocationSource::Route)
    }

    pub fn invoke_binding(&self, binding: UiEventBinding) -> UiInvocationResult {
        let key = binding.native_binding();
        let Some(route_id) = self.routes_by_binding.get(&key).copied() else {
            return UiInvocationResult::failure(
                None,
                Some(binding.clone()),
                UiInvocationError::UnknownBinding(key),
            );
        };
        let arguments = binding
            .action
            .as_ref()
            .map(|call| call.arguments.clone())
            .unwrap_or_default();
        self.invoke_route_internal(route_id, arguments, UiInvocationSource::Binding)
    }

    pub fn replace_tree(&mut self, snapshot: UiReflectionSnapshot) -> UiReflectionDiff {
        let tree_id = snapshot.tree_id.clone();
        let diff = if let Some(previous) = self.trees.insert(tree_id.clone(), snapshot) {
            compute_diff(&previous, self.trees.get(&tree_id).expect("replaced tree"))
        } else {
            UiReflectionDiff {
                tree_id: tree_id.clone(),
                changed_nodes: self
                    .trees
                    .get(&tree_id)
                    .map(|tree| tree.nodes.keys().copied().collect())
                    .unwrap_or_default(),
                removed_nodes: Vec::new(),
            }
        };
        self.rebuild_node_index();
        if !diff.is_empty() {
            self.broadcast(UiNotification::ReflectionDiff(diff.clone()));
        }
        diff
    }

    pub fn query_tree(&self, tree_id: &UiTreeId) -> Option<UiReflectionSnapshot> {
        self.trees.get(tree_id).cloned()
    }

    pub fn query_node(&self, node_path: &UiNodePath) -> Option<UiNodeDescriptor> {
        let (tree_id, node_id) = self.node_index.get(node_path)?;
        self.trees
            .get(tree_id)
            .and_then(|tree| tree.nodes.get(node_id))
            .cloned()
    }

    pub fn query_property(
        &self,
        node_path: &UiNodePath,
        property_name: &str,
    ) -> Option<UiPropertyDescriptor> {
        self.query_node(node_path)
            .and_then(|node| node.properties.get(property_name).cloned())
    }

    pub fn set_property(
        &mut self,
        node_path: UiNodePath,
        property_name: String,
        value: Value,
    ) -> Result<(), UiInvocationError> {
        let (tree_id, node_id) = self
            .node_index
            .get(&node_path)
            .cloned()
            .ok_or_else(|| UiInvocationError::UnknownNode(node_path.0.clone()))?;
        let tree = self
            .trees
            .get_mut(&tree_id)
            .ok_or_else(|| UiInvocationError::UnknownTree(tree_id.0.clone()))?;
        let node = tree
            .nodes
            .get_mut(&node_id)
            .ok_or_else(|| UiInvocationError::UnknownNode(node_path.0.clone()))?;
        let property = node.properties.get_mut(&property_name).ok_or_else(|| {
            UiInvocationError::UnknownProperty {
                node_path: node_path.0.clone(),
                property_name: property_name.clone(),
            }
        })?;
        if !property.writable {
            return Err(UiInvocationError::PropertyNotWritable {
                node_path: node_path.0.clone(),
                property_name,
            });
        }
        property.reflected_value = value;
        let diff = UiReflectionDiff {
            tree_id,
            changed_nodes: vec![node_id],
            removed_nodes: Vec::new(),
        };
        self.broadcast(UiNotification::ReflectionDiff(diff));
        Ok(())
    }

    pub fn call_action(
        &self,
        node_path: UiNodePath,
        action_id: String,
        arguments: Vec<UiBindingValue>,
    ) -> UiInvocationResult {
        let Some(node) = self.query_node(&node_path) else {
            return UiInvocationResult::failure(
                None,
                None,
                UiInvocationError::UnknownNode(node_path.0),
            );
        };
        let Some(action) = node.actions.get(&action_id) else {
            return UiInvocationResult::failure(
                None,
                None,
                UiInvocationError::UnknownAction {
                    node_path: node_path.0,
                    action_id,
                },
            );
        };
        if !action.callable_from_remote {
            return UiInvocationResult::failure(
                action.route_id,
                None,
                UiInvocationError::ActionNotCallable {
                    node_path: node.node_path.0,
                    action_id: action.action_id.clone(),
                },
            );
        }
        let Some(route_id) = action.route_id else {
            return UiInvocationResult::failure(
                None,
                None,
                UiInvocationError::ActionMissingRoute {
                    node_path: node.node_path.0,
                    action_id: action.action_id.clone(),
                },
            );
        };
        self.invoke_route_internal(
            route_id,
            arguments,
            UiInvocationSource::Action {
                node_path,
                action_id: action.action_id.clone(),
            },
        )
    }

    pub fn handle_request(&mut self, request: UiControlRequest) -> UiControlResponse {
        match request {
            UiControlRequest::InvokeBinding { binding } => {
                UiControlResponse::Invocation(self.invoke_binding(binding))
            }
            UiControlRequest::InvokeRoute {
                route_id,
                arguments,
            } => UiControlResponse::Invocation(self.invoke_route(route_id, arguments)),
            UiControlRequest::QueryTree { tree_id } => {
                UiControlResponse::Tree(self.query_tree(&tree_id))
            }
            UiControlRequest::QueryNode { node_path } => {
                UiControlResponse::Node(self.query_node(&node_path))
            }
            UiControlRequest::QueryProperty {
                node_path,
                property_name,
            } => UiControlResponse::Property(self.query_property(&node_path, &property_name)),
            UiControlRequest::SetProperty {
                node_path,
                property_name,
                value,
            } => match self.set_property(node_path, property_name, value) {
                Ok(()) => UiControlResponse::Ack,
                Err(error) => {
                    UiControlResponse::Invocation(UiInvocationResult::failure(None, None, error))
                }
            },
            UiControlRequest::CallAction {
                node_path,
                action_id,
                arguments,
            } => UiControlResponse::Invocation(self.call_action(node_path, action_id, arguments)),
            UiControlRequest::SubscribeDiffs => {
                let (subscription_id, _receiver) = self.subscribe();
                UiControlResponse::Subscription(subscription_id)
            }
            UiControlRequest::Unsubscribe { subscription_id } => {
                self.unsubscribe(subscription_id);
                UiControlResponse::Ack
            }
        }
    }

    fn invoke_route_internal(
        &self,
        route_id: UiRouteId,
        arguments: Vec<UiBindingValue>,
        source: UiInvocationSource,
    ) -> UiInvocationResult {
        let Some(route) = self.routes_by_id.get(&route_id) else {
            return UiInvocationResult::failure(
                Some(route_id),
                None,
                UiInvocationError::UnknownRoute(route_id),
            );
        };
        let effective_arguments = if arguments.is_empty() {
            route
                .binding
                .action
                .as_ref()
                .map(|call| call.arguments.clone())
                .unwrap_or_default()
        } else {
            arguments
        };
        let context = UiInvocationContext {
            route_id: route.route_id,
            binding: route.binding.clone(),
            arguments: effective_arguments,
            source,
        };
        let Some(handler) = &route.handler else {
            return UiInvocationResult::failure(
                Some(route.route_id),
                Some(route.binding.clone()),
                UiInvocationError::HandlerFailed(
                    "route has no execution handler registered".to_string(),
                ),
            );
        };
        let result = match handler(context) {
            Ok(value) => UiInvocationResult::success(route.route_id, route.binding.clone(), value),
            Err(error) => UiInvocationResult::failure(
                Some(route.route_id),
                Some(route.binding.clone()),
                error,
            ),
        };
        self.broadcast(UiNotification::Invocation(result.clone()));
        result
    }

    fn rebuild_node_index(&mut self) {
        self.node_index.clear();
        for (tree_id, tree) in &self.trees {
            for (node_id, node) in &tree.nodes {
                self.node_index
                    .insert(node.node_path.clone(), (tree_id.clone(), *node_id));
            }
        }
    }

    fn broadcast(&self, notification: UiNotification) {
        for sender in self.subscriptions.values() {
            let _ = sender.send(notification.clone());
        }
    }
}

fn compute_diff(
    previous: &UiReflectionSnapshot,
    current: &UiReflectionSnapshot,
) -> UiReflectionDiff {
    let mut changed_nodes = Vec::new();
    let mut removed_nodes = Vec::new();

    for (node_id, node) in &current.nodes {
        if previous.nodes.get(node_id) != Some(node) {
            changed_nodes.push(*node_id);
        }
    }
    for node_id in previous.nodes.keys() {
        if !current.nodes.contains_key(node_id) {
            removed_nodes.push(*node_id);
        }
    }

    UiReflectionDiff {
        tree_id: current.tree_id.clone(),
        changed_nodes,
        removed_nodes,
    }
}
