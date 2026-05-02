use std::collections::BTreeMap;

use crossbeam_channel::Sender;

use super::route_entry::RouteEntry;
use zircon_runtime_interface::ui::event_ui::{
    UiNodeId, UiNodePath, UiNotification, UiReflectionSnapshot, UiRouteId, UiSubscriptionId,
    UiTreeId,
};

#[derive(Default)]
pub struct UiEventManager {
    pub(super) next_route_id: u64,
    pub(super) next_subscription_id: u64,
    pub(super) routes_by_id: BTreeMap<UiRouteId, RouteEntry>,
    pub(super) routes_by_binding: BTreeMap<String, UiRouteId>,
    pub(super) trees: BTreeMap<UiTreeId, UiReflectionSnapshot>,
    pub(super) node_index: BTreeMap<UiNodePath, (UiTreeId, UiNodeId)>,
    pub(super) subscriptions: BTreeMap<UiSubscriptionId, Sender<UiNotification>>,
}
