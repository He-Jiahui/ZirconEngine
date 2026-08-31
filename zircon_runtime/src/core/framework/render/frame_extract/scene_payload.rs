use super::super::{EnvironmentExtract, SpriteExtract};
use super::{
    DebugOverlayExtract, GeometryExtract, LightingExtract, ParticleExtract, PostProcessExtract,
    RenderSharedSceneDomain, RenderSkeletalPoseExtract, RenderWorldSnapshotHandle, VisibilityInput,
};

/// Immutable-by-default scene data shared by every submission of one extract
/// generation. Each large domain performs copy-on-write independently.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderFrameScenePayload {
    pub world: RenderWorldSnapshotHandle,
    pub geometry: RenderSharedSceneDomain<GeometryExtract>,
    pub animation_poses: RenderSharedSceneDomain<Vec<RenderSkeletalPoseExtract>>,
    pub lighting: RenderSharedSceneDomain<LightingExtract>,
    pub environment: RenderSharedSceneDomain<EnvironmentExtract>,
    pub post_process: RenderSharedSceneDomain<PostProcessExtract>,
    pub debug: RenderSharedSceneDomain<DebugOverlayExtract>,
    pub sprites: RenderSharedSceneDomain<SpriteExtract>,
    pub particles: RenderSharedSceneDomain<ParticleExtract>,
    pub visibility: RenderSharedSceneDomain<VisibilityInput>,
}

impl RenderFrameScenePayload {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        world: RenderWorldSnapshotHandle,
        geometry: GeometryExtract,
        animation_poses: Vec<RenderSkeletalPoseExtract>,
        lighting: LightingExtract,
        environment: EnvironmentExtract,
        post_process: PostProcessExtract,
        debug: DebugOverlayExtract,
        sprites: SpriteExtract,
        particles: ParticleExtract,
        visibility: VisibilityInput,
    ) -> Self {
        Self {
            world,
            geometry: geometry.into(),
            animation_poses: animation_poses.into(),
            lighting: lighting.into(),
            environment: environment.into(),
            post_process: post_process.into(),
            debug: debug.into(),
            sprites: sprites.into(),
            particles: particles.into(),
            visibility: visibility.into(),
        }
    }

    pub fn shares_large_domains_with(&self, other: &Self) -> bool {
        self.geometry.ptr_eq(&other.geometry)
            && self.animation_poses.ptr_eq(&other.animation_poses)
            && self.lighting.ptr_eq(&other.lighting)
            && self.environment.ptr_eq(&other.environment)
            && self.post_process.ptr_eq(&other.post_process)
            && self.debug.ptr_eq(&other.debug)
            && self.sprites.ptr_eq(&other.sprites)
            && self.particles.ptr_eq(&other.particles)
            && self.visibility.ptr_eq(&other.visibility)
    }
}
