use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::core::resource::{ResourceId, ResourceMarker, UntypedResourceHandle};

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
