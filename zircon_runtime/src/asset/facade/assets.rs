use std::sync::Arc;

use super::{Asset, AssetEventReceiver, AssetLoadState, Handle};
use crate::core::resource::{
    ResourceLease, ResourceManager, ResourceMarker, ResourceMutationBatch,
    ResourceReadinessGenerationAssemblyExt, ResourceRecord, ResourceRegistryError, ResourceResult,
};

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
            .readiness_generation()
            .contains_kind(handle.id(), TAsset::Marker::KIND)
    }

    pub fn load_state(&self, handle: Handle<TAsset>) -> AssetLoadState {
        let generation = self.manager.readiness_generation();
        let Some(row) = generation.row(handle.id()) else {
            return AssetLoadState::NotLoaded;
        };
        if row.record.kind != TAsset::Marker::KIND {
            return AssetLoadState::NotLoaded;
        }
        row.typed_load_state::<TAsset>().into()
    }

    pub fn failure_reason(&self, handle: Handle<TAsset>) -> Option<String> {
        let generation = self.manager.readiness_generation();
        let row = generation.row(handle.id())?;
        if row.record.kind != TAsset::Marker::KIND {
            return None;
        }
        row.record.failure_reason().map(str::to_owned)
    }

    pub fn insert(&self, record: ResourceRecord, asset: TAsset) -> ResourceResult<Handle<TAsset>> {
        if record.kind != TAsset::Marker::KIND {
            return Err(ResourceRegistryError::KindConflict {
                id: record.id.to_string(),
                current_kind: record.kind,
                requested_kind: TAsset::Marker::KIND,
            });
        }
        let handle = self
            .manager
            .register_ready(record, asset)?
            .typed::<TAsset::Marker>()
            .expect("matching resource kind produces a typed handle");
        Ok(Handle::from_resource_handle(handle))
    }

    pub fn remove_by_locator(
        &self,
        locator: &crate::asset::AssetUri,
    ) -> ResourceResult<Option<ResourceRecord>> {
        let receipt = self.manager.commit(
            ResourceMutationBatch::new().remove_kind(locator.clone(), TAsset::Marker::KIND),
        )?;
        let removed = receipt.removed_records().next().cloned();
        Ok(removed)
    }

    pub fn subscribe_events(&self) -> AssetEventReceiver<TAsset> {
        super::typed_event_receiver(self.manager.subscribe())
    }
}
