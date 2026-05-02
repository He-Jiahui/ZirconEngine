use serde::{Deserialize, Serialize};

use super::{ResourceEventKind, ResourceId, ResourceLocator};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceEvent {
    pub kind: ResourceEventKind,
    pub id: ResourceId,
    pub locator: Option<ResourceLocator>,
    pub previous_locator: Option<ResourceLocator>,
    pub revision: u64,
}
