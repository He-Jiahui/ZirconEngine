use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::{UiBindingValue, UiEventBinding};

use super::{
    UiNodeDescriptor, UiNodePath, UiPropertyDescriptor, UiReflectionDiff, UiReflectionSnapshot,
};

macro_rules! define_u64_id {
    ($name:ident) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            Default,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
        )]
        pub struct $name(pub u64);

        impl $name {
            pub const fn new(value: u64) -> Self {
                Self(value)
            }
        }
    };
}

define_u64_id!(UiRouteId);
define_u64_id!(UiSubscriptionId);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiInvocationSource {
    Route,
    Binding,
    Action {
        node_path: UiNodePath,
        action_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiInvocationContext {
    pub route_id: UiRouteId,
    pub binding: UiEventBinding,
    pub arguments: Vec<UiBindingValue>,
    pub source: UiInvocationSource,
}

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiInvocationError {
    #[error("unknown binding {0}")]
    UnknownBinding(String),
    #[error("unknown route {0:?}")]
    UnknownRoute(UiRouteId),
    #[error("unknown tree {0}")]
    UnknownTree(String),
    #[error("unknown node {0}")]
    UnknownNode(String),
    #[error("unknown property {property_name} on {node_path}")]
    UnknownProperty {
        node_path: String,
        property_name: String,
    },
    #[error("property {property_name} on {node_path} is not writable")]
    PropertyNotWritable {
        node_path: String,
        property_name: String,
    },
    #[error("unknown action {action_id} on {node_path}")]
    UnknownAction {
        node_path: String,
        action_id: String,
    },
    #[error("action {action_id} on {node_path} is not callable remotely")]
    ActionNotCallable {
        node_path: String,
        action_id: String,
    },
    #[error("action {action_id} on {node_path} is missing a registered route")]
    ActionMissingRoute {
        node_path: String,
        action_id: String,
    },
    #[error("subscription {0:?} is missing")]
    MissingSubscription(UiSubscriptionId),
    #[error("handler failed: {0}")]
    HandlerFailed(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiInvocationResult {
    pub route_id: Option<UiRouteId>,
    pub binding: Option<UiEventBinding>,
    pub value: Option<Value>,
    pub error: Option<UiInvocationError>,
}

impl UiInvocationResult {
    pub fn success(route_id: UiRouteId, binding: UiEventBinding, value: Value) -> Self {
        Self {
            route_id: Some(route_id),
            binding: Some(binding),
            value: Some(value),
            error: None,
        }
    }

    pub fn failure(
        route_id: Option<UiRouteId>,
        binding: Option<UiEventBinding>,
        error: UiInvocationError,
    ) -> Self {
        Self {
            route_id,
            binding,
            value: None,
            error: Some(error),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiNotification {
    ReflectionDiff(UiReflectionDiff),
    Invocation(UiInvocationResult),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiControlRequest {
    InvokeBinding {
        binding: UiEventBinding,
    },
    InvokeRoute {
        route_id: UiRouteId,
        arguments: Vec<UiBindingValue>,
    },
    QueryTree {
        tree_id: super::UiTreeId,
    },
    QueryNode {
        node_path: UiNodePath,
    },
    QueryProperty {
        node_path: UiNodePath,
        property_name: String,
    },
    SetProperty {
        node_path: UiNodePath,
        property_name: String,
        value: Value,
    },
    CallAction {
        node_path: UiNodePath,
        action_id: String,
        arguments: Vec<UiBindingValue>,
    },
    SubscribeDiffs,
    Unsubscribe {
        subscription_id: UiSubscriptionId,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiControlResponse {
    Invocation(UiInvocationResult),
    Tree(Option<UiReflectionSnapshot>),
    Node(Option<UiNodeDescriptor>),
    Property(Option<UiPropertyDescriptor>),
    Subscription(UiSubscriptionId),
    Ack,
}

pub type UiInvocationRequest = UiControlRequest;
pub type UiInvocationResponse = UiControlResponse;
