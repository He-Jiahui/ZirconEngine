use crate::core::math::UVec2;
use crate::core::resource::ResourceId;

use crate::core::framework::animation::AnimationPoseOutput;
use crate::core::framework::scene::{EntityId, Mobility, WorldHandle};

use super::{
    DisplayMode, PreviewEnvironmentExtract, RenderBakedLightingExtract, RenderBloomSettings,
    RenderColorGradingSettings, RenderDirectionalLightSnapshot, RenderHybridGiExtract,
    RenderMeshSnapshot, RenderOverlayExtract, RenderParticleBoundsSnapshot,
    RenderParticleSpriteSnapshot, RenderPointLightSnapshot, RenderReflectionProbeSnapshot,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderSpotLightSnapshot,
    RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract, SceneViewportExtractRequest,
    ViewportCameraSnapshot,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderWorldSnapshotHandle(u64);

impl RenderWorldSnapshotHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl From<WorldHandle> for RenderWorldSnapshotHandle {
    fn from(value: WorldHandle) -> Self {
        Self::new(value.get())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderExtractContext {
    pub world: RenderWorldSnapshotHandle,
    pub request: SceneViewportExtractRequest,
}

impl RenderExtractContext {
    pub fn new(world: RenderWorldSnapshotHandle, request: SceneViewportExtractRequest) -> Self {
        Self { world, request }
    }
}

pub trait RenderExtractProducer {
    fn build_render_frame_extract(&self, context: &RenderExtractContext) -> RenderFrameExtract;
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderViewExtract {
    pub camera: ViewportCameraSnapshot,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct GeometryExtract {
    pub meshes: Vec<RenderMeshSnapshot>,
    pub virtual_geometry: Option<RenderVirtualGeometryExtract>,
    pub virtual_geometry_debug: Option<RenderVirtualGeometryDebugState>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct LightingExtract {
    pub directional_lights: Vec<RenderDirectionalLightSnapshot>,
    pub point_lights: Vec<RenderPointLightSnapshot>,
    pub spot_lights: Vec<RenderSpotLightSnapshot>,
    pub reflection_probes: Vec<RenderReflectionProbeSnapshot>,
    pub baked_lighting: Option<RenderBakedLightingExtract>,
    pub hybrid_global_illumination: Option<RenderHybridGiExtract>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PostProcessExtract {
    pub preview: PreviewEnvironmentExtract,
    pub display_mode: DisplayMode,
    pub bloom: RenderBloomSettings,
    pub color_grading: RenderColorGradingSettings,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DebugOverlayExtract {
    pub overlays: RenderOverlayExtract,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ParticleExtract {
    pub emitters: Vec<EntityId>,
    pub sprites: Vec<RenderParticleSpriteSnapshot>,
    pub bounds: Vec<RenderParticleBoundsSnapshot>,
    pub sort_camera_position: Option<crate::core::math::Vec3>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisibilityRenderableInput {
    pub entity: EntityId,
    pub mobility: Mobility,
    pub render_layer_mask: u32,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct VisibilityInput {
    pub renderable_entities: Vec<EntityId>,
    pub static_entities: Vec<EntityId>,
    pub dynamic_entities: Vec<EntityId>,
    pub renderables: Vec<VisibilityRenderableInput>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderSkeletalPoseExtract {
    pub entity: EntityId,
    pub skeleton: ResourceId,
    pub pose: AnimationPoseOutput,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderFrameExtract {
    pub world: RenderWorldSnapshotHandle,
    pub view: RenderViewExtract,
    pub geometry: GeometryExtract,
    pub animation_poses: Vec<RenderSkeletalPoseExtract>,
    pub lighting: LightingExtract,
    pub post_process: PostProcessExtract,
    pub debug: DebugOverlayExtract,
    pub particles: ParticleExtract,
    pub visibility: VisibilityInput,
}

impl RenderFrameExtract {
    pub fn from_snapshot(world: RenderWorldSnapshotHandle, snapshot: RenderSceneSnapshot) -> Self {
        let renderables = snapshot
            .scene
            .meshes
            .iter()
            .map(|mesh| VisibilityRenderableInput {
                entity: mesh.node_id,
                mobility: mesh.mobility,
                render_layer_mask: mesh.render_layer_mask,
            })
            .collect::<Vec<_>>();
        let renderable_entities = renderables
            .iter()
            .map(|entry| entry.entity)
            .collect::<Vec<_>>();
        let static_entities = renderables
            .iter()
            .filter(|entry| entry.mobility == Mobility::Static)
            .map(|entry| entry.entity)
            .collect::<Vec<_>>();
        let dynamic_entities = renderables
            .iter()
            .filter(|entry| entry.mobility == Mobility::Dynamic)
            .map(|entry| entry.entity)
            .collect::<Vec<_>>();

        Self {
            world,
            view: RenderViewExtract {
                camera: snapshot.scene.camera.clone(),
            },
            geometry: GeometryExtract {
                meshes: snapshot.scene.meshes.clone(),
                virtual_geometry: None,
                virtual_geometry_debug: snapshot.virtual_geometry_debug,
            },
            animation_poses: Vec::new(),
            lighting: LightingExtract {
                directional_lights: snapshot.scene.directional_lights.clone(),
                point_lights: snapshot.scene.point_lights.clone(),
                spot_lights: snapshot.scene.spot_lights.clone(),
                reflection_probes: Vec::new(),
                baked_lighting: None,
                hybrid_global_illumination: None,
            },
            post_process: PostProcessExtract {
                preview: snapshot.preview.clone(),
                display_mode: snapshot.overlays.display_mode,
                bloom: RenderBloomSettings::default(),
                color_grading: RenderColorGradingSettings::default(),
            },
            debug: DebugOverlayExtract {
                overlays: snapshot.overlays,
            },
            particles: ParticleExtract::default(),
            visibility: VisibilityInput {
                renderable_entities,
                static_entities,
                dynamic_entities,
                renderables,
            },
        }
    }

    pub fn to_scene_snapshot(&self) -> RenderSceneSnapshot {
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: self.view.camera.clone(),
                meshes: self.geometry.meshes.clone(),
                directional_lights: self.lighting.directional_lights.clone(),
                point_lights: self.lighting.point_lights.clone(),
                spot_lights: self.lighting.spot_lights.clone(),
            },
            overlays: self.debug.overlays.clone(),
            preview: self.post_process.preview.clone(),
            virtual_geometry_debug: self.geometry.virtual_geometry_debug,
        }
    }

    pub fn apply_viewport_size(&mut self, viewport_size: UVec2) {
        self.view.camera.apply_viewport_size(viewport_size);
    }

    pub fn with_viewport_size(mut self, viewport_size: UVec2) -> Self {
        self.apply_viewport_size(viewport_size);
        self
    }
}
