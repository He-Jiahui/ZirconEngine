use serde::{Deserialize, Serialize};

use crate::{ResourceHandle, ResourceId, ResourceKind, ResourceMarker};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UntypedResourceHandle {
    id: ResourceId,
    kind: ResourceKind,
}

impl UntypedResourceHandle {
    pub fn new(id: ResourceId, kind: ResourceKind) -> Self {
        Self { id, kind }
    }

    pub fn id(self) -> ResourceId {
        self.id
    }

    pub fn kind(self) -> ResourceKind {
        self.kind
    }

    pub fn typed<TMarker: ResourceMarker>(self) -> Option<ResourceHandle<TMarker>> {
        (self.kind == TMarker::KIND).then(|| ResourceHandle::new(self.id))
    }
}
