use crate::core::CoreError;
use crate::core::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker,
    AnimationSkeletonMarker, AnimationStateMachineMarker, MaterialMarker, ModelMarker,
    PhysicsMaterialMarker, ResourceHandle, SceneMarker, ShaderMarker, TextureMarker,
    UiLayoutMarker, UiStyleMarker, UiWidgetMarker,
};

use super::super::ProjectAssetManager;
use crate::asset::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset, AssetId, MaterialAsset, ModelAsset, PhysicsMaterialAsset,
    SceneAsset, ShaderAsset, TextureAsset, UiLayoutAsset, UiStyleAsset, UiWidgetAsset,
};

impl ProjectAssetManager {
    pub fn load_model_asset(&self, id: AssetId) -> Result<ModelAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<ModelMarker>::new(id), "model")
    }

    pub fn load_material_asset(&self, id: AssetId) -> Result<MaterialAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<MaterialMarker>::new(id), "material")
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

    pub fn load_texture_asset(&self, id: AssetId) -> Result<TextureAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<TextureMarker>::new(id), "texture")
    }

    pub fn load_shader_asset(&self, id: AssetId) -> Result<ShaderAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<ShaderMarker>::new(id), "shader")
    }

    pub fn load_scene_asset(&self, id: AssetId) -> Result<SceneAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<SceneMarker>::new(id), "scene")
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
