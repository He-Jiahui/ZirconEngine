use crate::core::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, DataMarker, FontMarker, MaterialGraphMarker, MaterialMarker,
    ModelMarker, NavMeshMarker, NavigationSettingsMarker, PhysicsMaterialMarker, PrefabMarker,
    ResourceHandle, SceneMarker, ShaderMarker, SoundMarker, TerrainLayerStackMarker, TerrainMarker,
    TextureMarker, TileMapMarker, TileSetMarker, UiLayoutMarker, UiStyleMarker, UiWidgetMarker,
};
use crate::core::CoreError;

use super::super::ProjectAssetManager;
use crate::asset::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset, AssetId, DataAsset, FontAsset, MaterialAsset, MaterialGraphAsset,
    ModelAsset, NavMeshAsset, NavigationSettingsAsset, PhysicsMaterialAsset, PrefabAsset,
    SceneAsset, ShaderAsset, SoundAsset, TerrainAsset, TerrainLayerStackAsset, TextureAsset,
    TileMapAsset, TileSetAsset, UiLayoutAsset, UiStyleAsset, UiWidgetAsset,
};

impl ProjectAssetManager {
    pub fn load_model_asset(&self, id: AssetId) -> Result<ModelAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<ModelMarker>::new(id), "model")
    }

    pub fn load_material_asset(&self, id: AssetId) -> Result<MaterialAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<MaterialMarker>::new(id), "material")
    }

    pub fn load_material_graph_asset(&self, id: AssetId) -> Result<MaterialGraphAsset, CoreError> {
        self.load_typed(
            id,
            ResourceHandle::<MaterialGraphMarker>::new(id),
            "material graph",
        )
    }

    pub fn load_data_asset(&self, id: AssetId) -> Result<DataAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<DataMarker>::new(id), "data")
    }

    pub fn load_physics_material_asset(
        &self,
        id: AssetId,
    ) -> Result<PhysicsMaterialAsset, CoreError> {
        self.load_typed(
            id,
            ResourceHandle::<PhysicsMaterialMarker>::new(id),
            "physics material",
        )
    }

    pub fn load_nav_mesh_asset(&self, id: AssetId) -> Result<NavMeshAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<NavMeshMarker>::new(id), "nav mesh")
    }

    pub fn load_navigation_settings_asset(
        &self,
        id: AssetId,
    ) -> Result<NavigationSettingsAsset, CoreError> {
        self.load_typed(
            id,
            ResourceHandle::<NavigationSettingsMarker>::new(id),
            "navigation settings",
        )
    }

    pub fn load_terrain_asset(&self, id: AssetId) -> Result<TerrainAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<TerrainMarker>::new(id), "terrain")
    }

    pub fn load_terrain_layer_stack_asset(
        &self,
        id: AssetId,
    ) -> Result<TerrainLayerStackAsset, CoreError> {
        self.load_typed(
            id,
            ResourceHandle::<TerrainLayerStackMarker>::new(id),
            "terrain layer stack",
        )
    }

    pub fn load_tile_set_asset(&self, id: AssetId) -> Result<TileSetAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<TileSetMarker>::new(id), "tile set")
    }

    pub fn load_tile_map_asset(&self, id: AssetId) -> Result<TileMapAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<TileMapMarker>::new(id), "tile map")
    }

    pub fn load_prefab_asset(&self, id: AssetId) -> Result<PrefabAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<PrefabMarker>::new(id), "prefab")
    }

    pub fn load_texture_asset(&self, id: AssetId) -> Result<TextureAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<TextureMarker>::new(id), "texture")
    }

    pub fn load_shader_asset(&self, id: AssetId) -> Result<ShaderAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<ShaderMarker>::new(id), "shader")
    }

    pub fn load_scene_asset(&self, id: AssetId) -> Result<SceneAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<SceneMarker>::new(id), "scene")
    }

    pub fn load_sound_asset(&self, id: AssetId) -> Result<SoundAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<SoundMarker>::new(id), "sound")
    }

    pub fn load_font_asset(&self, id: AssetId) -> Result<FontAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<FontMarker>::new(id), "font")
    }

    pub fn load_animation_skeleton_asset(
        &self,
        id: AssetId,
    ) -> Result<AnimationSkeletonAsset, CoreError> {
        self.load_typed(
            id,
            ResourceHandle::<AnimationSkeletonMarker>::new(id),
            "animation skeleton",
        )
    }

    pub fn load_animation_clip_asset(&self, id: AssetId) -> Result<AnimationClipAsset, CoreError> {
        self.load_typed(
            id,
            ResourceHandle::<AnimationClipMarker>::new(id),
            "animation clip",
        )
    }

    pub fn load_animation_sequence_asset(
        &self,
        id: AssetId,
    ) -> Result<AnimationSequenceAsset, CoreError> {
        self.load_typed(
            id,
            ResourceHandle::<AnimationSequenceMarker>::new(id),
            "animation sequence",
        )
    }

    pub fn load_animation_graph_asset(
        &self,
        id: AssetId,
    ) -> Result<AnimationGraphAsset, CoreError> {
        self.load_typed(
            id,
            ResourceHandle::<AnimationGraphMarker>::new(id),
            "animation graph",
        )
    }

    pub fn load_animation_state_machine_asset(
        &self,
        id: AssetId,
    ) -> Result<AnimationStateMachineAsset, CoreError> {
        self.load_typed(
            id,
            ResourceHandle::<AnimationStateMachineMarker>::new(id),
            "animation state machine",
        )
    }

    pub fn load_ui_layout_asset(&self, id: AssetId) -> Result<UiLayoutAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<UiLayoutMarker>::new(id), "ui layout")
    }

    pub fn load_ui_widget_asset(&self, id: AssetId) -> Result<UiWidgetAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<UiWidgetMarker>::new(id), "ui widget")
    }

    pub fn load_ui_style_asset(&self, id: AssetId) -> Result<UiStyleAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<UiStyleMarker>::new(id), "ui style")
    }
}
