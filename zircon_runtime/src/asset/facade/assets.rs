use std::sync::Arc;

use super::{Asset, AssetEventReceiver, AssetLoadState, Handle};
use crate::core::resource::{ResourceLease, ResourceManager, ResourceMarker, ResourceRecord};

#[derive(Clone, Debug)]
pub struct Assets<TAsset: Asset> {
    manager: ResourceManager,
    asset: std::marker::PhantomData<TAsset>,
}

impl<TAsset: Asset> Assets<TAsset> {
    pub fn new(manager: ResourceManager) -> Self {
        Self {
            manager,
            asset: std::marker::PhantomData,
        }
    }

    pub fn get(&self, handle: Handle<TAsset>) -> Option<Arc<TAsset>> {
        self.manager
            .get::<TAsset::Marker, TAsset>(handle.resource_handle())
    }

    pub fn get_cloned(&self, handle: Handle<TAsset>) -> Option<TAsset> {
        self.get(handle).map(|asset| asset.as_ref().clone())
    }

    pub fn acquire(&self, handle: Handle<TAsset>) -> Option<ResourceLease<TAsset>> {
        self.manager
            .acquire::<TAsset::Marker, TAsset>(handle.resource_handle())
    }

    pub fn contains(&self, handle: Handle<TAsset>) -> bool {
        self.manager
            .registry()
            .get(handle.id())
            .is_some_and(|record| record.kind == TAsset::Marker::KIND)
    }

    pub fn load_state(&self, handle: Handle<TAsset>) -> AssetLoadState {
        let record = self.manager.registry().get(handle.id()).cloned();
        if !record
            .as_ref()
            .is_some_and(|record| record.kind == TAsset::Marker::KIND)
        {
            return AssetLoadState::NotLoaded;
        }

        AssetLoadState::from_resource(
            record.as_ref(),
            self.manager.runtime_state(handle.id()),
            self.get(handle).is_some(),
        )
    }

    pub fn insert(&self, record: ResourceRecord, asset: TAsset) -> Option<Handle<TAsset>> {
        if record.kind != TAsset::Marker::KIND {
            return None;
        }
        self.manager
            .register_ready(record, asset)
            .typed::<TAsset::Marker>()
            .map(Handle::from_resource_handle)
    }

    pub fn remove_by_locator(&self, locator: &crate::asset::AssetUri) -> Option<ResourceRecord> {
        let record = self.manager.registry().get_by_locator(locator).cloned()?;
        (record.kind == TAsset::Marker::KIND)
            .then(|| self.manager.remove_by_locator(locator))
            .flatten()
    }

    pub fn subscribe_events(&self) -> AssetEventReceiver<TAsset> {
        super::typed_event_receiver(self.manager.subscribe())
    }
}
