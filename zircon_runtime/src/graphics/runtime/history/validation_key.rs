use crate::core::framework::render::{
    LightingExtract, ParticleExtract, PostProcessExtract, RenderFrameExtract,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::math::{Transform, Vec4};
use crate::core::resource::ResourceId;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FrameHistoryValidationKey {
    // Reuse temporal history only when the frame inputs that can affect scene color match.
    world: RenderWorldSnapshotHandle,
    camera: ViewportCameraSnapshot,
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
            camera: ViewportCameraSnapshot::default(),
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
    render_layer_mask: u32,
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
        Self {
            world: extract.world,
            camera: extract.view.camera.clone(),
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
                    render_layer_mask: mesh.render_layer_mask,
                })
                .collect(),
            lighting: extract.lighting.clone(),
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
