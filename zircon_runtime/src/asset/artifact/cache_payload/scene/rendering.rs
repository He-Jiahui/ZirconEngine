use serde::{Deserialize, Serialize};

use crate::asset::AssetReference;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheSceneMeshPrimitiveBindingAsset {
    mesh: AssetReference,
    material: AssetReference,
}

impl From<&crate::asset::SceneMeshPrimitiveBindingAsset>
    for ArtifactCacheSceneMeshPrimitiveBindingAsset
{
    fn from(asset: &crate::asset::SceneMeshPrimitiveBindingAsset) -> Self {
        Self {
            mesh: asset.mesh.clone(),
            material: asset.material.clone(),
        }
    }
}

impl From<ArtifactCacheSceneMeshPrimitiveBindingAsset>
    for crate::asset::SceneMeshPrimitiveBindingAsset
{
    fn from(asset: ArtifactCacheSceneMeshPrimitiveBindingAsset) -> Self {
        Self {
            mesh: asset.mesh,
            material: asset.material,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheSceneMeshLodLevelAsset {
    min_distance: crate::core::math::Real,
    model: AssetReference,
    mesh: Option<AssetReference>,
    material: AssetReference,
    primitives: Vec<ArtifactCacheSceneMeshPrimitiveBindingAsset>,
}

impl From<&crate::asset::SceneMeshLodLevelAsset> for ArtifactCacheSceneMeshLodLevelAsset {
    fn from(asset: &crate::asset::SceneMeshLodLevelAsset) -> Self {
        Self {
            min_distance: asset.min_distance,
            model: asset.model.clone(),
            mesh: asset.mesh.clone(),
            material: asset.material.clone(),
            primitives: asset
                .primitives
                .iter()
                .map(ArtifactCacheSceneMeshPrimitiveBindingAsset::from)
                .collect(),
        }
    }
}

impl ArtifactCacheSceneMeshLodLevelAsset {
    fn into_asset(self) -> crate::asset::SceneMeshLodLevelAsset {
        crate::asset::SceneMeshLodLevelAsset {
            min_distance: self.min_distance,
            model: self.model,
            mesh: self.mesh,
            material: self.material,
            primitives: self
                .primitives
                .into_iter()
                .map(crate::asset::SceneMeshPrimitiveBindingAsset::from)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheSceneMeshInstanceAsset {
    model: AssetReference,
    mesh: Option<AssetReference>,
    material: AssetReference,
    render_queue: i32,
    material_queue: i32,
    order_in_layer: i32,
    depth_bias: crate::core::math::Real,
    morph_weights: Vec<crate::core::math::Real>,
    primitives: Vec<ArtifactCacheSceneMeshPrimitiveBindingAsset>,
    lods: Vec<ArtifactCacheSceneMeshLodLevelAsset>,
}

impl From<&crate::asset::SceneMeshInstanceAsset> for ArtifactCacheSceneMeshInstanceAsset {
    fn from(asset: &crate::asset::SceneMeshInstanceAsset) -> Self {
        Self {
            model: asset.model.clone(),
            mesh: asset.mesh.clone(),
            material: asset.material.clone(),
            render_queue: asset.render_queue,
            material_queue: asset.material_queue,
            order_in_layer: asset.order_in_layer,
            depth_bias: asset.depth_bias,
            morph_weights: asset.morph_weights.clone(),
            primitives: asset
                .primitives
                .iter()
                .map(ArtifactCacheSceneMeshPrimitiveBindingAsset::from)
                .collect(),
            lods: asset
                .lods
                .iter()
                .map(ArtifactCacheSceneMeshLodLevelAsset::from)
                .collect(),
        }
    }
}

impl ArtifactCacheSceneMeshInstanceAsset {
    pub(super) fn into_asset(self) -> crate::asset::SceneMeshInstanceAsset {
        crate::asset::SceneMeshInstanceAsset {
            model: self.model,
            mesh: self.mesh,
            material: self.material,
            render_queue: self.render_queue,
            material_queue: self.material_queue,
            order_in_layer: self.order_in_layer,
            depth_bias: self.depth_bias,
            morph_weights: self.morph_weights,
            primitives: self
                .primitives
                .into_iter()
                .map(crate::asset::SceneMeshPrimitiveBindingAsset::from)
                .collect(),
            lods: self
                .lods
                .into_iter()
                .map(ArtifactCacheSceneMeshLodLevelAsset::into_asset)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheSceneCameraAsset {
    #[serde(default)]
    core_pipeline: crate::core::framework::render::CorePipelineKind,
    projection_mode: crate::core::framework::render::ProjectionMode,
    fov_y_radians: crate::core::math::Real,
    ortho_size: crate::core::math::Real,
    z_near: crate::core::math::Real,
    z_far: crate::core::math::Real,
    target: ArtifactCacheSceneCameraTargetAsset,
    viewport: Option<crate::asset::SceneViewportRectAsset>,
    order: i32,
    active: bool,
    hdr: bool,
    exposure_ev100: crate::core::math::Real,
    clear_color: crate::core::framework::render::RenderCameraClearColor,
    msaa_samples: u32,
    post_process_settings: Option<crate::asset::ScenePostProcessSettingsAsset>,
}

impl From<&crate::asset::SceneCameraAsset> for ArtifactCacheSceneCameraAsset {
    fn from(asset: &crate::asset::SceneCameraAsset) -> Self {
        Self {
            core_pipeline: asset.core_pipeline,
            projection_mode: asset.projection_mode,
            fov_y_radians: asset.fov_y_radians,
            ortho_size: asset.ortho_size,
            z_near: asset.z_near,
            z_far: asset.z_far,
            target: ArtifactCacheSceneCameraTargetAsset::from(&asset.target),
            viewport: asset.viewport,
            order: asset.order,
            active: asset.active,
            hdr: asset.hdr,
            exposure_ev100: asset.exposure_ev100,
            clear_color: asset.clear_color,
            msaa_samples: asset.msaa_samples,
            post_process_settings: asset.post_process_settings,
        }
    }
}

impl ArtifactCacheSceneCameraAsset {
    pub(super) fn into_asset(self) -> crate::asset::SceneCameraAsset {
        crate::asset::SceneCameraAsset {
            core_pipeline: self.core_pipeline,
            projection_mode: self.projection_mode,
            fov_y_radians: self.fov_y_radians,
            ortho_size: self.ortho_size,
            z_near: self.z_near,
            z_far: self.z_far,
            target: self.target.into_asset(),
            viewport: self.viewport,
            order: self.order,
            active: self.active,
            hdr: self.hdr,
            exposure_ev100: self.exposure_ev100,
            clear_color: self.clear_color,
            msaa_samples: self.msaa_samples,
            post_process_settings: self.post_process_settings,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ArtifactCacheSceneCameraTargetAsset {
    PrimarySurface,
    Texture { texture: AssetReference },
    Headless { size: [u32; 2] },
}

impl From<&crate::asset::SceneCameraTargetAsset> for ArtifactCacheSceneCameraTargetAsset {
    fn from(target: &crate::asset::SceneCameraTargetAsset) -> Self {
        match target {
            crate::asset::SceneCameraTargetAsset::PrimarySurface => Self::PrimarySurface,
            crate::asset::SceneCameraTargetAsset::Texture { texture } => Self::Texture {
                texture: texture.clone(),
            },
            crate::asset::SceneCameraTargetAsset::Headless { size } => {
                Self::Headless { size: *size }
            }
        }
    }
}

impl ArtifactCacheSceneCameraTargetAsset {
    fn into_asset(self) -> crate::asset::SceneCameraTargetAsset {
        match self {
            Self::PrimarySurface => crate::asset::SceneCameraTargetAsset::PrimarySurface,
            Self::Texture { texture } => crate::asset::SceneCameraTargetAsset::Texture { texture },
            Self::Headless { size } => crate::asset::SceneCameraTargetAsset::Headless { size },
        }
    }
}
