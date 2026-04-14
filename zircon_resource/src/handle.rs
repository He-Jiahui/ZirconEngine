use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::{ResourceId, ResourceKind, ResourceMarker};

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

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceHandle<TMarker> {
    id: ResourceId,
    #[serde(skip)]
    marker: PhantomData<TMarker>,
}

impl<TMarker> Copy for ResourceHandle<TMarker> {}

impl<TMarker> Clone for ResourceHandle<TMarker> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<TMarker> ResourceHandle<TMarker> {
    pub fn new(id: ResourceId) -> Self {
        Self {
            id,
            marker: PhantomData,
        }
    }

    pub fn id(self) -> ResourceId {
        self.id
    }
}

impl<TMarker: ResourceMarker> From<ResourceHandle<TMarker>> for UntypedResourceHandle {
    fn from(value: ResourceHandle<TMarker>) -> Self {
        Self::new(value.id, TMarker::KIND)
    }
}
