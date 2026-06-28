use crate::core::framework::render::{
    CameraRenderDescriptor, LightingExtract, ParticleExtract, PostProcessExtract,
    RenderFrameExtract, RenderHybridGiExtract, RenderLayerSet, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::math::{Transform, Vec4};
use crate::core::resource::ResourceId;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FrameHistoryValidationKey {
    // Reuse temporal history only when the frame inputs that can affect scene color match.
    world: RenderWorldSnapshotHandle,
    camera: CameraRenderDescriptor,
    meshes: Vec<FrameHistoryMeshValidationKey>,
    lighting: LightingExtract,
    animation_poses: Vec<FrameHistoryAnimationPoseValidationKey>,
    post_process: PostProcessExtract,
    particles: ParticleExtract,
    effective_features: Vec<String>,
}

impl Default for FrameHistoryValidationKey {
    fn default() -> Self {
        Self {
            world: RenderWorldSnapshotHandle::new(0),
            camera: CameraRenderDescriptor::from_camera_payload(
                None,
                ViewportCameraSnapshot::default(),
            ),
            meshes: Vec::new(),
            lighting: LightingExtract::default(),
            animation_poses: Vec::new(),
            post_process: PostProcessExtract::default(),
            particles: ParticleExtract::default(),
            effective_features: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct FrameHistoryMeshValidationKey {
    entity: EntityId,
    transform: Transform,
    model: ResourceId,
    material: ResourceId,
    tint: Vec4,
    mobility: Mobility,
    render_layer_mask: RenderLayerSet,
}

#[derive(Clone, Debug, PartialEq)]
struct FrameHistoryAnimationPoseValidationKey {
    entity: EntityId,
    skeleton: ResourceId,
    pose: crate::core::framework::animation::AnimationPoseOutput,
}

impl FrameHistoryValidationKey {
    pub(crate) fn from_extract(
        extract: &RenderFrameExtract,
        effective_features: Vec<String>,
    ) -> Self {
        Self::from_extract_with_hybrid_gi(
            extract,
            effective_features,
            extract.lighting.hybrid_global_illumination.as_ref(),
        )
    }

    pub(crate) fn from_extract_with_hybrid_gi(
        extract: &RenderFrameExtract,
        effective_features: Vec<String>,
        hybrid_global_illumination: Option<&RenderHybridGiExtract>,
    ) -> Self {
        let mut lighting = extract.lighting.clone();
        lighting.hybrid_global_illumination = hybrid_global_illumination.cloned();
        Self {
            world: extract.world,
            camera: extract
                .view
                .selected_camera_descriptor()
                .cloned()
                .unwrap_or_else(|| {
                    CameraRenderDescriptor::from_camera_payload(None, extract.view.camera.clone())
                }),
            meshes: extract
                .geometry
                .meshes
                .iter()
                .map(|mesh| FrameHistoryMeshValidationKey {
                    entity: mesh.node_id,
                    transform: mesh.transform,
                    model: mesh.model.id(),
                    material: mesh.material.id(),
                    tint: mesh.tint,
                    mobility: mesh.mobility,
                    render_layer_mask: mesh.render_layer_mask.clone(),
                })
                .collect(),
            lighting,
            animation_poses: extract
                .animation_poses
                .iter()
                .map(|pose| FrameHistoryAnimationPoseValidationKey {
                    entity: pose.entity,
                    skeleton: pose.skeleton,
                    pose: pose.pose.clone(),
                })
                .collect(),
            post_process: extract.post_process.clone(),
            particles: extract.particles.clone(),
            effective_features,
        }
    }
}
