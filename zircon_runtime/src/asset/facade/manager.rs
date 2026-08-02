use std::sync::Arc;

use super::{
    Asset, AssetEventReceiver, AssetLoadState, AssetLoadStates, Assets, DependencyLoadState,
    Handle, RecursiveDependencyLoadState,
};
use crate::asset::ProjectAssetManager;
use crate::asset::{AssetId, AssetUri};
use crate::core::resource::{
    ResourceHandle, ResourceMarker, ResourceReadinessGeneration, ResourceState,
};
use crate::core::{CoreError, CoreError::Initialization};

fn asset_error_message(message: impl Into<String>) -> CoreError {
    Initialization(
        crate::asset::PROJECT_ASSET_MANAGER_NAME.to_string(),
        message.into(),
    )
}

impl ProjectAssetManager {
    pub fn load<TAsset: Asset>(&self, locator: &AssetUri) -> Result<Handle<TAsset>, CoreError> {
        let record = self
            .resource_manager()
            .registry()
            .get_by_locator(locator)
            .cloned()
            .ok_or_else(|| asset_error_message(format!("missing asset locator {locator}")))?;
        if record.kind != TAsset::Marker::KIND {
            return Err(asset_error_message(format!(
                "asset {locator} was {:?}, not {:?}",
                record.kind,
                TAsset::Marker::KIND
            )));
        }

        self.ensure_loaded::<TAsset>(record.id)?;
        Ok(Handle::new(record.id))
    }

    pub fn handle<TAsset: Asset>(&self, locator: &AssetUri) -> Result<Handle<TAsset>, CoreError> {
        let record = self
            .resource_manager()
            .registry()
            .get_by_locator(locator)
            .cloned()
            .ok_or_else(|| asset_error_message(format!("missing asset locator {locator}")))?;
        if record.kind != TAsset::Marker::KIND {
            return Err(asset_error_message(format!(
                "asset {locator} was {:?}, not {:?}",
                record.kind,
                TAsset::Marker::KIND
            )));
        }
        Ok(Handle::new(record.id))
    }

    pub fn assets<TAsset: Asset>(&self) -> Assets<TAsset> {
        Assets::new(self.resource_manager())
    }

    pub fn load_state<TAsset: Asset>(&self, handle: Handle<TAsset>) -> AssetLoadState {
        self.load_states(handle).load_state
    }

    pub fn failure_reason<TAsset: Asset>(&self, handle: Handle<TAsset>) -> Option<String> {
        self.assets::<TAsset>().failure_reason(handle)
    }

    pub fn dependency_load_state<TAsset: Asset>(
        &self,
        handle: Handle<TAsset>,
    ) -> DependencyLoadState {
        self.load_states(handle).dependency_load_state
    }

    pub fn load_states<TAsset: Asset>(&self, handle: Handle<TAsset>) -> AssetLoadStates {
        let generation = self.resource_manager().readiness_generation();
        self.load_states_from_generation(handle, &generation)
    }

    pub fn is_loaded<TAsset: Asset>(&self, handle: Handle<TAsset>) -> bool {
        self.load_state(handle).is_loaded()
    }

    pub fn is_loaded_with_direct_dependencies<TAsset: Asset>(
        &self,
        handle: Handle<TAsset>,
    ) -> bool {
        self.load_states(handle)
            .is_loaded_with_direct_dependencies()
    }

    pub fn is_loaded_with_dependencies<TAsset: Asset>(&self, handle: Handle<TAsset>) -> bool {
        self.load_states(handle).is_loaded_with_dependencies()
    }

    pub fn recursive_dependency_load_state<TAsset: Asset>(
        &self,
        handle: Handle<TAsset>,
    ) -> RecursiveDependencyLoadState {
        self.load_states(handle).recursive_dependency_load_state
    }

    pub fn asset_load_state_by_id<TAsset: Asset>(&self, id: AssetId) -> AssetLoadState {
        self.load_state(Handle::<TAsset>::new(id))
    }

    pub fn subscribe_asset_events<TAsset: Asset>(&self) -> AssetEventReceiver<TAsset> {
        self.assets::<TAsset>().subscribe_events()
    }

    pub(super) fn load_states_from_generation<TAsset: Asset>(
        &self,
        handle: Handle<TAsset>,
        generation: &Arc<ResourceReadinessGeneration>,
    ) -> AssetLoadStates {
        let Some(row) = generation.row(handle.id()) else {
            return not_loaded_states();
        };
        if row.record.kind != TAsset::Marker::KIND {
            return not_loaded_states();
        }

        let load_state = AssetLoadState::from(row.typed_load_state::<TAsset>());
        AssetLoadStates {
            dependency_load_state: row.direct_dependency_state.into(),
            recursive_dependency_load_state: if load_state.is_loaded() {
                row.recursive_dependency_state.into()
            } else {
                load_state.clone().into()
            },
            load_state,
        }
    }

    fn ensure_loaded<TAsset: Asset>(&self, id: AssetId) -> Result<TAsset, CoreError> {
        let handle = ResourceHandle::<TAsset::Marker>::new(id);
        if let Some(asset) = self
            .resource_manager()
            .get::<TAsset::Marker, TAsset>(handle)
        {
            return Ok(asset.as_ref().clone());
        }

        let record = self
            .resource_manager()
            .registry()
            .get(id)
            .cloned()
            .ok_or_else(|| {
                asset_error_message(format!("missing resource record for asset id {id}"))
            })?;
        if record.kind != TAsset::Marker::KIND {
            return Err(asset_error_message(format!(
                "asset {id} was {:?}, not {:?}",
                record.kind,
                TAsset::Marker::KIND
            )));
        }
        if record.state != ResourceState::Ready {
            return Err(asset_error_message(format!(
                "asset {id} is {:?}, not ready",
                record.state
            )));
        }

        self.ensure_resident(id)?;
        self.resource_manager()
            .get::<TAsset::Marker, TAsset>(handle)
            .map(|asset| asset.as_ref().clone())
            .ok_or_else(|| {
                asset_error_message(format!(
                    "asset {id} was not a ready typed facade payload {}",
                    TAsset::LABEL
                ))
            })
    }
}

fn not_loaded_states() -> AssetLoadStates {
    AssetLoadStates {
        load_state: AssetLoadState::NotLoaded,
        dependency_load_state: DependencyLoadState::NotLoaded,
        recursive_dependency_load_state: RecursiveDependencyLoadState::NotLoaded,
    }
}
