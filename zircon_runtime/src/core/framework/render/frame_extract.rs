use crate::core::math::UVec2;
use crate::core::resource::ResourceId;

use crate::core::framework::animation::AnimationPoseOutput;
use crate::core::framework::scene::{EntityId, Mobility, WorldHandle};

use super::{
    build_mesh_phase_queue, build_sprite_phase_queue, AntiAliasSettings, CorePipelineKind,
    DisplayMode, FallbackSkyboxKind, MeshPhaseInput, PostProcessPassGraph,
    PostProcessStackDescriptor, PreviewEnvironmentExtract, RenderAmbientLightSnapshot,
    RenderBakedLightingExtract, RenderBloomSettings, RenderColorGradingSettings,
    RenderDirectionalLightSnapshot, RenderHybridGiExtract, RenderMaterialAlphaMode,
    RenderMeshSnapshot, RenderOverlayExtract, RenderParticleBoundsSnapshot,
    RenderParticleSpriteSnapshot, RenderPhaseQueue, RenderPointLightSnapshot,
    RenderRectLightSnapshot, RenderReflectionProbeSnapshot, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderSpotLightSnapshot, RenderSpriteSnapshot,
    RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract, SceneViewportExtractRequest,
    SpriteExtract, SpritePhaseInput, ViewportCameraSnapshot,
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
    pub anti_alias: AntiAliasSettings,
}

impl RenderViewExtract {
    pub fn from_camera(camera: ViewportCameraSnapshot) -> Self {
        let core_pipeline = camera.core_pipeline_kind();
        let anti_alias = AntiAliasSettings::from_camera_msaa_samples(camera.msaa_samples);
        Self {
            camera,
            core_pipeline,
            anti_alias,
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

#[derive(Clone, Debug, PartialEq)]
pub struct SpritePhaseExtractInput {
    pub entity: EntityId,
    pub sprite_index: usize,
    pub material_alpha_mode: RenderMaterialAlphaMode,
    pub z_order: i32,
    pub depth: f32,
}

impl SpritePhaseExtractInput {
    pub fn new(
        entity: EntityId,
        sprite_index: usize,
        material_alpha_mode: RenderMaterialAlphaMode,
        z_order: i32,
        depth: f32,
    ) -> Self {
        Self {
            entity,
            sprite_index,
            material_alpha_mode,
            z_order,
            depth,
        }
    }
}

impl SpriteExtract {
    pub fn from_sprites(
        core_pipeline: CorePipelineKind,
        sprites: Vec<RenderSpriteSnapshot>,
    ) -> Self {
        let phase_inputs = sprites
            .iter()
            .enumerate()
            .map(|(sprite_index, sprite)| {
                SpritePhaseExtractInput::new(
                    sprite.entity,
                    sprite_index,
                    sprite.material_alpha_mode,
                    sprite.z_order,
                    sprite.transform.translation.z,
                )
            })
            .collect::<Vec<_>>();
        Self::from_sprites_and_phase_inputs(core_pipeline, sprites, phase_inputs)
    }

    pub fn from_sprites_and_phase_inputs(
        core_pipeline: CorePipelineKind,
        sprites: Vec<RenderSpriteSnapshot>,
        phase_inputs: Vec<SpritePhaseExtractInput>,
    ) -> Self {
        let phase_queue = build_sprite_phase_queue(
            core_pipeline,
            phase_inputs.iter().map(|input| SpritePhaseInput {
                entity: input.entity,
                sprite_index: input.sprite_index,
                material_alpha_mode: input.material_alpha_mode,
                z_order: input.z_order,
                depth: input.depth,
            }),
        );

        Self {
            sprites,
            phase_queue,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct LightingExtract {
    pub directional_lights: Vec<RenderDirectionalLightSnapshot>,
    pub point_lights: Vec<RenderPointLightSnapshot>,
    pub spot_lights: Vec<RenderSpotLightSnapshot>,
    pub ambient_lights: Vec<RenderAmbientLightSnapshot>,
    pub rect_lights: Vec<RenderRectLightSnapshot>,
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
    pub stack: PostProcessStackDescriptor,
    pub graph: PostProcessPassGraph,
}

impl Default for PostProcessExtract {
    fn default() -> Self {
        let bloom = RenderBloomSettings::default();
        let color_grading = RenderColorGradingSettings::default();
        Self::from_parts(
            PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: crate::core::math::Vec4::ZERO,
            },
            DisplayMode::Shaded,
            bloom,
            color_grading,
            false,
            false,
        )
    }
}

impl PostProcessExtract {
    pub fn from_parts(
        preview: PreviewEnvironmentExtract,
        display_mode: DisplayMode,
        bloom: RenderBloomSettings,
        color_grading: RenderColorGradingSettings,
        history_resolve_enabled: bool,
        history_available: bool,
    ) -> Self {
        let stack = PostProcessStackDescriptor::from_extract_settings(
            &bloom,
            &color_grading,
            history_resolve_enabled,
            history_available,
        );
        let graph = stack.validated_graph();
        Self {
            preview,
            display_mode,
            bloom,
            color_grading,
            stack,
            graph,
        }
    }

    pub fn rebuild_graph(&mut self, history_resolve_enabled: bool, history_available: bool) {
        self.stack = PostProcessStackDescriptor::from_extract_settings(
            &self.bloom,
            &self.color_grading,
            history_resolve_enabled,
            history_available,
        );
        self.graph = self.stack.validated_graph();
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
    pub sprites: SpriteExtract,
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
                ambient_lights: snapshot.scene.ambient_lights.clone(),
                rect_lights: snapshot.scene.rect_lights.clone(),
                reflection_probes: Vec::new(),
                baked_lighting: None,
                hybrid_global_illumination: None,
            },
            post_process: PostProcessExtract::from_parts(
                snapshot.preview.clone(),
                snapshot.overlays.display_mode,
                RenderBloomSettings::default(),
                RenderColorGradingSettings::default(),
                false,
                false,
            ),
            debug: DebugOverlayExtract {
                overlays: snapshot.overlays,
            },
            sprites: SpriteExtract::default(),
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
                ambient_lights: self.lighting.ambient_lights.clone(),
                rect_lights: self.lighting.rect_lights.clone(),
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
