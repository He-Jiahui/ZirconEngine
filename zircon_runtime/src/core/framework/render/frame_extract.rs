//! 场景世界在提交 GPU 工作前，把视图、几何、光照和变化集投影为帧快照。
//! 下游可见性与渲染器消费同一抽取结果，避免各阶段重新读取可变世界状态。
#[path = "frame_extract/camera_target_size.rs"]
mod camera_target_size;
#[path = "frame_extract/debug_overlay.rs"]
mod debug_overlay;
#[path = "frame_extract/extract_context.rs"]
mod extract_context;
#[path = "frame_extract/extract_producer.rs"]
mod extract_producer;
#[path = "frame_extract/frame.rs"]
mod frame;
#[path = "frame_extract/geometry.rs"]
mod geometry;
#[path = "frame_extract/lighting.rs"]
mod lighting;
#[path = "frame_extract/particle.rs"]
mod particle;
#[path = "../../render/frame_extract/particle_extract_policy.rs"]
mod particle_extract_policy;
#[path = "frame_extract/particle_gpu_frame.rs"]
mod particle_gpu_frame;
#[path = "frame_extract/phase_queue.rs"]
mod phase_queue;
#[path = "frame_extract/post_process.rs"]
mod post_process;
#[path = "frame_extract/scene_changes/mod.rs"]
mod scene_changes;
#[path = "frame_extract/scene_payload.rs"]
mod scene_payload;
#[path = "frame_extract/shared_scene_domain.rs"]
mod shared_scene_domain;
#[path = "frame_extract/skeletal_pose.rs"]
mod skeletal_pose;
#[path = "frame_extract/sprite_extract.rs"]
mod sprite_extract;
#[path = "frame_extract/sprite_phase_input.rs"]
mod sprite_phase_input;
#[path = "frame_extract/view.rs"]
mod view;
#[path = "frame_extract/visibility.rs"]
mod visibility;
#[path = "frame_extract/visibility_renderable.rs"]
mod visibility_renderable;
#[path = "frame_extract/world_snapshot_handle.rs"]
mod world_snapshot_handle;

pub use debug_overlay::DebugOverlayExtract;
pub use extract_context::RenderExtractContext;
pub use extract_producer::RenderExtractProducer;
pub use frame::{RenderFrameExtract, RenderFrameTiming};
pub use geometry::{GeometryExtract, GeometryPhaseInput, StaticMeshBatchExtract};
pub use lighting::LightingExtract;
pub use particle::ParticleExtract;
pub use particle_gpu_frame::RenderParticleGpuFrameExtract;
pub use post_process::PostProcessExtract;
pub use scene_changes::{
    RenderComponentChangeArtifact, RenderComponentChangeKind, RenderComponentChangeMask,
    RenderComponentChangeStats, RenderComponentFullReprojectionReason, RenderComponentMeshLodLevel,
    RenderComponentMeshPayload, RenderComponentMeshPrimitiveBinding, RenderComponentProjectionMode,
    RenderComponentSnapshot, RenderComponentSourceWorldId, RenderComponentValue,
};
pub use scene_payload::RenderFrameScenePayload;
pub use shared_scene_domain::RenderSharedSceneDomain;
pub use skeletal_pose::RenderSkeletalPoseExtract;
pub use sprite_phase_input::SpritePhaseExtractInput;
pub use view::RenderViewExtract;
pub use visibility::VisibilityInput;
pub use visibility_renderable::VisibilityRenderableInput;
pub use world_snapshot_handle::RenderWorldSnapshotHandle;

pub(super) use camera_target_size::camera_target_size_from_descriptor;
pub(super) use phase_queue::resolved_phase_queue;

#[cfg(test)]
#[path = "frame_extract/tests/cases.rs"]
mod tests;
