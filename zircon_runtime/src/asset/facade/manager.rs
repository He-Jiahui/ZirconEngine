use std::collections::HashSet;

use super::{
    Asset, AssetEventReceiver, AssetLoadState, AssetLoadStates, Assets, DependencyLoadState,
    Handle, RecursiveDependencyLoadState,
};
use crate::asset::ProjectAssetManager;
use crate::asset::{AssetId, AssetUri};
use crate::core::resource::{ResourceHandle, ResourceMarker, ResourceRecord, ResourceState};
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
        self.assets::<TAsset>().load_state(handle)
    }

    pub fn dependency_load_state<TAsset: Asset>(
        &self,
        handle: Handle<TAsset>,
    ) -> DependencyLoadState {
        let record = self.resource_manager().registry().get(handle.id()).cloned();
        let Some(record) = record else {
            return DependencyLoadState::NotLoaded;
        };
        if record.kind != TAsset::Marker::KIND {
            return DependencyLoadState::NotLoaded;
        }

        self.aggregate_direct_dependency_state(&record.dependency_ids)
            .unwrap_or(DependencyLoadState::Loaded)
    }

    pub fn load_states<TAsset: Asset>(&self, handle: Handle<TAsset>) -> AssetLoadStates {
        self.load_states_from_root(handle)
    }

    pub fn is_loaded<TAsset: Asset>(&self, handle: Handle<TAsset>) -> bool {
        self.load_states(handle).is_loaded()
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
        let root_state = self.load_state(handle);
        if !matches!(root_state, AssetLoadState::Loaded) {
            return root_state.into();
        }
        let mut visited = HashSet::new();
        let dependency_state = self.aggregate_dependency_state(handle.id(), &mut visited);
        dependency_state.unwrap_or(RecursiveDependencyLoadState::Loaded)
    }

    pub fn asset_load_state_by_id<TAsset: Asset>(&self, id: AssetId) -> AssetLoadState {
        self.load_state(Handle::<TAsset>::new(id))
    }

    pub fn subscribe_asset_events<TAsset: Asset>(&self) -> AssetEventReceiver<TAsset> {
        self.assets::<TAsset>().subscribe_events()
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

    fn load_states_from_root<TAsset: Asset>(&self, handle: Handle<TAsset>) -> AssetLoadStates {
        let record = self.resource_manager().registry().get(handle.id()).cloned();
        let Some(record) = record else {
            return not_loaded_states();
        };
        if record.kind != TAsset::Marker::KIND {
            return not_loaded_states();
        }

        let load_state = AssetLoadState::from_resource(
            Some(&record),
            self.resource_manager().runtime_state(record.id),
            self.resource_manager()
                .get::<TAsset::Marker, TAsset>(handle.resource_handle())
                .is_some(),
        );
        let dependency_load_state = self
            .aggregate_direct_dependency_state(&record.dependency_ids)
            .unwrap_or(DependencyLoadState::Loaded);
        let recursive_dependency_load_state = if matches!(load_state, AssetLoadState::Loaded) {
            let mut visited = HashSet::new();
            self.aggregate_dependency_state_from_ids(
                record.id,
                &record.dependency_ids,
                &mut visited,
            )
            .unwrap_or(RecursiveDependencyLoadState::Loaded)
        } else {
            load_state.clone().into()
        };

        AssetLoadStates {
            load_state,
            dependency_load_state,
            recursive_dependency_load_state,
        }
    }

    fn aggregate_dependency_state(
        &self,
        id: AssetId,
        visited: &mut HashSet<AssetId>,
    ) -> Option<RecursiveDependencyLoadState> {
        let record = self.resource_manager().registry().get(id).cloned()?;
        self.aggregate_dependency_state_from_ids(id, &record.dependency_ids, visited)
    }

    fn aggregate_dependency_state_from_ids(
        &self,
        id: AssetId,
        dependency_ids: &[AssetId],
        visited: &mut HashSet<AssetId>,
    ) -> Option<RecursiveDependencyLoadState> {
        if !visited.insert(id) {
            return None;
        }
        let mut aggregate = None;
        for dependency_id in dependency_ids {
            let dependency = self
                .resource_manager()
                .registry()
                .get(*dependency_id)
                .cloned();
            let dependency_state = self.dependency_record_load_state(dependency.as_ref());
            aggregate = combine_recursive_dependency_state(aggregate, dependency_state.into());
            if let Some(nested) = self.aggregate_dependency_state(*dependency_id, visited) {
                aggregate = combine_recursive_dependency_state(aggregate, nested);
            }
        }
        aggregate
    }

    fn aggregate_direct_dependency_state(
        &self,
        dependency_ids: &[AssetId],
    ) -> Option<DependencyLoadState> {
        let mut aggregate = None;
        for dependency_id in dependency_ids {
            let dependency = self
                .resource_manager()
                .registry()
                .get(*dependency_id)
                .cloned();
            let dependency_state = self.dependency_record_load_state(dependency.as_ref());
            aggregate = combine_dependency_state(aggregate, dependency_state.into());
        }
        aggregate
    }

    fn dependency_record_load_state(&self, record: Option<&ResourceRecord>) -> AssetLoadState {
        let Some(record) = record else {
            return AssetLoadState::Failed;
        };
        AssetLoadState::from_resource(
            Some(record),
            self.resource_manager().runtime_state(record.id()),
            self.resource_manager().get_untyped(record.id()).is_some(),
        )
    }
}

fn not_loaded_states() -> AssetLoadStates {
    AssetLoadStates {
        load_state: AssetLoadState::NotLoaded,
        dependency_load_state: DependencyLoadState::NotLoaded,
        recursive_dependency_load_state: RecursiveDependencyLoadState::NotLoaded,
    }
}

fn combine_dependency_state(
    current: Option<DependencyLoadState>,
    next: DependencyLoadState,
) -> Option<DependencyLoadState> {
    Some(match current {
        Some(current) if dependency_rank(&current) >= dependency_rank(&next) => current,
        _ => next,
    })
}

fn dependency_rank(state: &DependencyLoadState) -> u8 {
    match state {
        DependencyLoadState::Loaded => 0,
        DependencyLoadState::NotLoaded => 1,
        DependencyLoadState::Loading => 2,
        DependencyLoadState::Reloading => 3,
        DependencyLoadState::Failed => 4,
    }
}

fn combine_recursive_dependency_state(
    current: Option<RecursiveDependencyLoadState>,
    next: RecursiveDependencyLoadState,
) -> Option<RecursiveDependencyLoadState> {
    Some(match current {
        Some(current)
            if recursive_dependency_rank(&current) >= recursive_dependency_rank(&next) =>
        {
            current
        }
        _ => next,
    })
}

fn recursive_dependency_rank(state: &RecursiveDependencyLoadState) -> u8 {
    match state {
        RecursiveDependencyLoadState::Loaded => 0,
        RecursiveDependencyLoadState::NotLoaded => 1,
        RecursiveDependencyLoadState::Loading => 2,
        RecursiveDependencyLoadState::Reloading => 3,
        RecursiveDependencyLoadState::Failed => 4,
    }
}
