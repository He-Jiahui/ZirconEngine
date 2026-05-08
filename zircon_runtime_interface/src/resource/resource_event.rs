use serde::{Deserialize, Serialize};

use super::{ResourceEventKind, ResourceId, ResourceKind, ResourceLocator};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceEvent {
    pub kind: ResourceEventKind,
    pub resource_kind: ResourceKind,
    pub id: ResourceId,
    pub locator: Option<ResourceLocator>,
    pub previous_locator: Option<ResourceLocator>,
    pub revision: u64,
}
