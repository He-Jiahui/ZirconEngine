use super::Asset;
use crate::asset::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset, DataAsset, FontAsset, MaterialAsset, MaterialGraphAsset,
    ModelAsset, NavMeshAsset, NavigationSettingsAsset, PhysicsMaterialAsset, PrefabAsset,
    SceneAsset, ShaderAsset, SoundAsset, TerrainAsset, TerrainLayerStackAsset, TextureAsset,
    TileMapAsset, TileSetAsset, UiLayoutAsset, UiStyleAsset, UiV2ComponentAsset, UiV2StyleAsset,
    UiV2ViewAsset, UiWidgetAsset,
};
use crate::core::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, DataMarker, FontMarker, MaterialGraphMarker, MaterialMarker,
    ModelMarker, NavMeshMarker, NavigationSettingsMarker, PhysicsMaterialMarker, PrefabMarker,
    SceneMarker, ShaderMarker, SoundMarker, TerrainLayerStackMarker, TerrainMarker, TextureMarker,
    TileMapMarker, TileSetMarker, UiLayoutMarker, UiStyleMarker, UiWidgetMarker,
};

macro_rules! impl_asset {
    ($asset:ty, $marker:ty, $label:literal) => {
        impl Asset for $asset {
            type Marker = $marker;

            const LABEL: &'static str = $label;
        }
    };
}

impl_asset!(DataAsset, DataMarker, "data");
impl_asset!(TextureAsset, TextureMarker, "texture");
impl_asset!(ShaderAsset, ShaderMarker, "shader");
impl_asset!(MaterialAsset, MaterialMarker, "material");
impl_asset!(MaterialGraphAsset, MaterialGraphMarker, "material_graph");
impl_asset!(SoundAsset, SoundMarker, "sound");
impl_asset!(FontAsset, FontMarker, "font");
impl_asset!(
    PhysicsMaterialAsset,
    PhysicsMaterialMarker,
    "physics_material"
);
impl_asset!(NavMeshAsset, NavMeshMarker, "nav_mesh");
impl_asset!(
    NavigationSettingsAsset,
    NavigationSettingsMarker,
    "navigation_settings"
);
impl_asset!(TerrainAsset, TerrainMarker, "terrain");
impl_asset!(
    TerrainLayerStackAsset,
    TerrainLayerStackMarker,
    "terrain_layer_stack"
);
impl_asset!(TileSetAsset, TileSetMarker, "tile_set");
impl_asset!(TileMapAsset, TileMapMarker, "tile_map");
impl_asset!(PrefabAsset, PrefabMarker, "prefab");
impl_asset!(SceneAsset, SceneMarker, "scene");
impl_asset!(ModelAsset, ModelMarker, "model");
impl_asset!(
    AnimationSkeletonAsset,
    AnimationSkeletonMarker,
    "animation_skeleton"
);
impl_asset!(AnimationClipAsset, AnimationClipMarker, "animation_clip");
impl_asset!(
    AnimationSequenceAsset,
    AnimationSequenceMarker,
    "animation_sequence"
);
impl_asset!(AnimationGraphAsset, AnimationGraphMarker, "animation_graph");
impl_asset!(
    AnimationStateMachineAsset,
    AnimationStateMachineMarker,
    "animation_state_machine"
);
impl_asset!(UiLayoutAsset, UiLayoutMarker, "ui_layout");
impl_asset!(UiWidgetAsset, UiWidgetMarker, "ui_widget");
impl_asset!(UiStyleAsset, UiStyleMarker, "ui_style");
impl_asset!(UiV2ViewAsset, UiLayoutMarker, "ui_v2_view");
impl_asset!(UiV2ComponentAsset, UiWidgetMarker, "ui_v2_component");
impl_asset!(UiV2StyleAsset, UiStyleMarker, "ui_v2_style");
