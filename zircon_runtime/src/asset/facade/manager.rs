use std::collections::HashSet;

use super::{
    Asset, AssetEventReceiver, AssetLoadState, Assets, Handle, RecursiveDependencyLoadState,
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

        match TAsset::Marker::KIND {
            crate::asset::AssetKind::Data => self.load_data_asset(id).and_then(downcast_asset),
            crate::asset::AssetKind::Texture => {
                self.load_texture_asset(id).and_then(downcast_asset)
            }
            crate::asset::AssetKind::Shader => self.load_shader_asset(id).and_then(downcast_asset),
            crate::asset::AssetKind::Material => {
                self.load_material_asset(id).and_then(downcast_asset)
            }
            crate::asset::AssetKind::MaterialGraph => {
                self.load_material_graph_asset(id).and_then(downcast_asset)
            }
            crate::asset::AssetKind::Sound => self.load_sound_asset(id).and_then(downcast_asset),
            crate::asset::AssetKind::Font => self.load_font_asset(id).and_then(downcast_asset),
            crate::asset::AssetKind::PhysicsMaterial => self
                .load_physics_material_asset(id)
                .and_then(downcast_asset),
            crate::asset::AssetKind::NavMesh => {
                self.load_nav_mesh_asset(id).and_then(downcast_asset)
            }
            crate::asset::AssetKind::NavigationSettings => self
                .load_navigation_settings_asset(id)
                .and_then(downcast_asset),
            crate::asset::AssetKind::Terrain => {
                self.load_terrain_asset(id).and_then(downcast_asset)
            }
            crate::asset::AssetKind::TerrainLayerStack => self
                .load_terrain_layer_stack_asset(id)
                .and_then(downcast_asset),
            crate::asset::AssetKind::TileSet => {
                self.load_tile_set_asset(id).and_then(downcast_asset)
            }
            crate::asset::AssetKind::TileMap => {
                self.load_tile_map_asset(id).and_then(downcast_asset)
            }
            crate::asset::AssetKind::Prefab => self.load_prefab_asset(id).and_then(downcast_asset),
            crate::asset::AssetKind::Scene => self.load_scene_asset(id).and_then(downcast_asset),
            crate::asset::AssetKind::Model => self.load_model_asset(id).and_then(downcast_asset),
            crate::asset::AssetKind::AnimationSkeleton => self
                .load_animation_skeleton_asset(id)
                .and_then(downcast_asset),
            crate::asset::AssetKind::AnimationClip => {
                self.load_animation_clip_asset(id).and_then(downcast_asset)
            }
            crate::asset::AssetKind::AnimationSequence => self
                .load_animation_sequence_asset(id)
                .and_then(downcast_asset),
            crate::asset::AssetKind::AnimationGraph => {
                self.load_animation_graph_asset(id).and_then(downcast_asset)
            }
            crate::asset::AssetKind::AnimationStateMachine => self
                .load_animation_state_machine_asset(id)
                .and_then(downcast_asset),
            crate::asset::AssetKind::UiLayout => {
                self.load_ui_layout_asset(id).and_then(downcast_asset)
            }
            crate::asset::AssetKind::UiWidget => {
                self.load_ui_widget_asset(id).and_then(downcast_asset)
            }
            crate::asset::AssetKind::UiStyle => {
                self.load_ui_style_asset(id).and_then(downcast_asset)
            }
        }
    }

    fn aggregate_dependency_state(
        &self,
        id: AssetId,
        visited: &mut HashSet<AssetId>,
    ) -> Option<RecursiveDependencyLoadState> {
        if !visited.insert(id) {
            return None;
        }
        let record = self.resource_manager().registry().get(id).cloned()?;
        let mut aggregate = None;
        for dependency_id in record.dependency_ids {
            let dependency = self
                .resource_manager()
                .registry()
                .get(dependency_id)
                .cloned();
            let dependency_state = self.dependency_record_load_state(dependency.as_ref());
            aggregate = combine_recursive_dependency_state(aggregate, dependency_state.into());
            if let Some(nested) = self.aggregate_dependency_state(dependency_id, visited) {
                aggregate = combine_recursive_dependency_state(aggregate, nested);
            }
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

fn downcast_asset<TAsset, TLoaded>(loaded: TLoaded) -> Result<TAsset, CoreError>
where
    TAsset: Asset,
    TLoaded: crate::core::resource::ResourceData,
{
    let boxed: Box<dyn std::any::Any> = Box::new(loaded);
    boxed.downcast::<TAsset>().map(|asset| *asset).map_err(|_| {
        asset_error_message(format!(
            "loaded {:?} asset could not be downcast to typed facade payload",
            TAsset::Marker::KIND
        ))
    })
}
