use crate::core::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, FontMarker, MaterialMarker, ModelMarker, PhysicsMaterialMarker,
    ResourceHandle, ResourceLease, SceneMarker, ShaderMarker, SoundMarker, TextureMarker,
    UiLayoutMarker, UiStyleMarker, UiWidgetMarker,
};
use crate::core::CoreError;

use super::super::ProjectAssetManager;
use crate::asset::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset, AssetId, FontAsset, MaterialAsset, ModelAsset,
    PhysicsMaterialAsset, SceneAsset, ShaderAsset, SoundAsset, TextureAsset, UiLayoutAsset,
    UiStyleAsset, UiWidgetAsset,
};

impl ProjectAssetManager {
    pub fn acquire_model_asset(&self, id: AssetId) -> Result<ResourceLease<ModelAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<ModelMarker>::new(id), "model")
    }

    pub fn acquire_material_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<MaterialAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<MaterialMarker>::new(id), "material")
    }

    pub fn acquire_physics_material_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<PhysicsMaterialAsset>, CoreError> {
        self.acquire_typed(
            id,
            ResourceHandle::<PhysicsMaterialMarker>::new(id),
            "physics material",
        )
    }

    pub fn acquire_texture_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<TextureAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<TextureMarker>::new(id), "texture")
    }

    pub fn acquire_shader_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<ShaderAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<ShaderMarker>::new(id), "shader")
    }

    pub fn acquire_scene_asset(&self, id: AssetId) -> Result<ResourceLease<SceneAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<SceneMarker>::new(id), "scene")
    }

    pub fn acquire_sound_asset(&self, id: AssetId) -> Result<ResourceLease<SoundAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<SoundMarker>::new(id), "sound")
    }

    pub fn acquire_font_asset(&self, id: AssetId) -> Result<ResourceLease<FontAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<FontMarker>::new(id), "font")
    }

    pub fn acquire_animation_skeleton_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<AnimationSkeletonAsset>, CoreError> {
        self.acquire_typed(
            id,
            ResourceHandle::<AnimationSkeletonMarker>::new(id),
            "animation skeleton",
        )
    }

    pub fn acquire_animation_clip_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<AnimationClipAsset>, CoreError> {
        self.acquire_typed(
            id,
            ResourceHandle::<AnimationClipMarker>::new(id),
            "animation clip",
        )
    }

    pub fn acquire_animation_sequence_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<AnimationSequenceAsset>, CoreError> {
        self.acquire_typed(
            id,
            ResourceHandle::<AnimationSequenceMarker>::new(id),
            "animation sequence",
        )
    }

    pub fn acquire_animation_graph_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<AnimationGraphAsset>, CoreError> {
        self.acquire_typed(
            id,
            ResourceHandle::<AnimationGraphMarker>::new(id),
            "animation graph",
        )
    }

    pub fn acquire_animation_state_machine_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<AnimationStateMachineAsset>, CoreError> {
        self.acquire_typed(
            id,
            ResourceHandle::<AnimationStateMachineMarker>::new(id),
            "animation state machine",
        )
    }

    pub fn acquire_ui_layout_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<UiLayoutAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<UiLayoutMarker>::new(id), "ui layout")
    }

    pub fn acquire_ui_widget_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<UiWidgetAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<UiWidgetMarker>::new(id), "ui widget")
    }

    pub fn acquire_ui_style_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<UiStyleAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<UiStyleMarker>::new(id), "ui style")
    }
}
