use crate::core::math::UVec2;
use crate::core::resource::ResourceId;

use crate::core::framework::animation::AnimationPoseOutput;
use crate::core::framework::scene::{EntityId, Mobility, WorldHandle};

use super::{
    build_mesh_phase_queue, CorePipelineKind, DisplayMode, FallbackSkyboxKind, MeshPhaseInput,
    PreviewEnvironmentExtract, RenderBakedLightingExtract, RenderBloomSettings,
    RenderColorGradingSettings, RenderDirectionalLightSnapshot, RenderHybridGiExtract,
    RenderMaterialAlphaMode, RenderMeshSnapshot, RenderOverlayExtract,
    RenderParticleBoundsSnapshot, RenderParticleSpriteSnapshot, RenderPhaseQueue,
    RenderPointLightSnapshot, RenderReflectionProbeSnapshot, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderSpotLightSnapshot, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExtract, SceneViewportExtractRequest, ViewportCameraSnapshot,
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
    pub core_pipeline: CorePipelineKind,
}

impl RenderViewExtract {
    pub fn from_camera(camera: ViewportCameraSnapshot) -> Self {
        let core_pipeline = camera.core_pipeline_kind();
        Self {
            camera,
            core_pipeline,
        }
    }
}

impl From<ViewportCameraSnapshot> for RenderViewExtract {
    fn from(camera: ViewportCameraSnapshot) -> Self {
        Self::from_camera(camera)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeometryPhaseInput {
    pub entity: EntityId,
    pub mesh_index: usize,
    pub material_alpha_mode: RenderMaterialAlphaMode,
    pub depth: f32,
}

impl GeometryPhaseInput {
    pub fn new(
        entity: EntityId,
        mesh_index: usize,
        material_alpha_mode: RenderMaterialAlphaMode,
        depth: f32,
    ) -> Self {
        Self {
            entity,
            mesh_index,
            material_alpha_mode,
            depth,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct GeometryExtract {
    pub meshes: Vec<RenderMeshSnapshot>,
    pub phase_inputs: Vec<GeometryPhaseInput>,
    pub phase_queue: RenderPhaseQueue,
    pub virtual_geometry: Option<RenderVirtualGeometryExtract>,
    pub virtual_geometry_debug: Option<RenderVirtualGeometryDebugState>,
}

impl GeometryExtract {
    pub fn from_meshes(core_pipeline: CorePipelineKind, meshes: Vec<RenderMeshSnapshot>) -> Self {
        let phase_inputs = meshes
            .iter()
            .enumerate()
            .map(|(mesh_index, mesh)| GeometryPhaseInput {
                entity: mesh.node_id,
                mesh_index,
                material_alpha_mode: RenderMaterialAlphaMode::Opaque,
                depth: mesh.transform.translation.z,
            })
            .collect::<Vec<_>>();
        Self::from_meshes_and_phase_inputs(core_pipeline, meshes, phase_inputs)
    }

    pub fn from_meshes_and_phase_inputs(
        core_pipeline: CorePipelineKind,
        meshes: Vec<RenderMeshSnapshot>,
        phase_inputs: Vec<GeometryPhaseInput>,
    ) -> Self {
        let phase_queue = build_mesh_phase_queue(
            core_pipeline,
            phase_inputs.iter().map(|input| MeshPhaseInput {
                entity: input.entity,
                mesh_index: input.mesh_index,
                material_alpha_mode: &input.material_alpha_mode,
                depth: input.depth,
            }),
        );

        Self {
            meshes,
            phase_inputs,
            phase_queue,
            virtual_geometry: None,
            virtual_geometry_debug: None,
        }
    }

    pub fn rebuild_phase_queue(&mut self, core_pipeline: CorePipelineKind) {
        self.phase_queue = build_mesh_phase_queue(
            core_pipeline,
            self.phase_inputs.iter().map(|input| MeshPhaseInput {
                entity: input.entity,
                mesh_index: input.mesh_index,
                material_alpha_mode: &input.material_alpha_mode,
                depth: input.depth,
            }),
        );
    }
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

impl Default for PostProcessExtract {
    fn default() -> Self {
        Self {
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: crate::core::math::Vec4::ZERO,
            },
            display_mode: DisplayMode::Shaded,
            bloom: RenderBloomSettings::default(),
            color_grading: RenderColorGradingSettings::default(),
        }
    }
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
    pub gpu_frame: Option<RenderParticleGpuFrameExtract>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct RenderParticleGpuFrameExtract {
    pub alive_count: u32,
    pub spawned_total: u32,
    pub per_emitter_spawned: Vec<u32>,
    pub indirect_draw_args: [u32; 4],
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
            view: RenderViewExtract::from_camera(snapshot.scene.camera.clone()),
            geometry: {
                let mut geometry = GeometryExtract::from_meshes(
                    snapshot.scene.camera.core_pipeline_kind(),
                    snapshot.scene.meshes.clone(),
                );
                geometry.virtual_geometry_debug = snapshot.virtual_geometry_debug;
                geometry
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
