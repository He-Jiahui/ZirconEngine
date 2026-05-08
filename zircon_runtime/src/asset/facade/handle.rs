use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Asset;
use crate::core::resource::{
    ResourceHandle, ResourceId, ResourceKind, ResourceMarker, UntypedResourceHandle,
};

pub struct Handle<TAsset: Asset> {
    id: ResourceId,
    asset: PhantomData<TAsset>,
}

impl<TAsset: Asset> Copy for Handle<TAsset> {}

impl<TAsset: Asset> Clone for Handle<TAsset> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<TAsset: Asset> Handle<TAsset> {
    pub fn new(id: ResourceId) -> Self {
        Self::from_resource_handle(ResourceHandle::new(id))
    }

    pub fn from_resource_handle(raw: ResourceHandle<TAsset::Marker>) -> Self {
        Self {
            id: raw.id(),
            asset: PhantomData,
        }
    }

    pub fn id(self) -> ResourceId {
        self.id
    }

    pub fn kind(self) -> ResourceKind {
        TAsset::Marker::KIND
    }

    pub fn resource_handle(self) -> ResourceHandle<TAsset::Marker> {
        ResourceHandle::new(self.id)
    }

    pub fn untyped(self) -> UntypedResourceHandle {
        self.resource_handle().into()
    }
}

impl<TAsset: Asset> From<Handle<TAsset>> for ResourceHandle<TAsset::Marker> {
    fn from(value: Handle<TAsset>) -> Self {
        value.resource_handle()
    }
}

impl<TAsset: Asset> From<Handle<TAsset>> for UntypedResourceHandle {
    fn from(value: Handle<TAsset>) -> Self {
        value.untyped()
    }
}

impl<TAsset: Asset> Debug for Handle<TAsset> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("id", &self.id)
            .field("kind", &TAsset::Marker::KIND)
            .finish()
    }
}

impl<TAsset: Asset> PartialEq for Handle<TAsset> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<TAsset: Asset> Eq for Handle<TAsset> {}

impl<TAsset: Asset> Hash for Handle<TAsset> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<TAsset: Asset> Serialize for Handle<TAsset> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.id.serialize(serializer)
    }
}

impl<'de, TAsset: Asset> Deserialize<'de> for Handle<TAsset> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        ResourceId::deserialize(deserializer).map(Self::new)
    }
}

impl<TAsset: Asset> TryFrom<UntypedResourceHandle> for Handle<TAsset> {
    type Error = UntypedResourceHandle;

    fn try_from(value: UntypedResourceHandle) -> Result<Self, Self::Error> {
        value
            .typed::<TAsset::Marker>()
            .map(Self::from_resource_handle)
            .ok_or(value)
    }
}
