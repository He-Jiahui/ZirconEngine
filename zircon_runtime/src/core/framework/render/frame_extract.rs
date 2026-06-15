use crate::core::math::UVec2;
use crate::core::resource::{MaterialMarker, MeshMarker, ModelMarker, ResourceHandle, ResourceId};
use std::collections::BTreeMap;

use crate::core::framework::animation::AnimationPoseOutput;
use crate::core::framework::scene::{EntityId, Mobility, WorldHandle};

mod particle_extract_policy;

use super::{
    build_mesh_phase_queue, build_sprite_phase_queue, AntiAliasSettings, CorePipelineKind,
    DisplayMode, FallbackSkyboxKind, MeshPhaseInput, PostProcessPassGraph,
    PostProcessStackDescriptor, PostProcessVolumeExtract, PreviewEnvironmentExtract,
    RenderAmbientLightSnapshot, RenderBakedLightingExtract, RenderBloomSettings,
    RenderCameraOrderReport, RenderCameraTarget, RenderColorGradingSettings,
    RenderDirectionalLightSnapshot, RenderExposureSettings, RenderFramePhaseQueueSummary,
    RenderHybridGiExtract, RenderLayerSet, RenderMaterialAlphaMode, RenderMeshSnapshot,
    RenderOverlayExtract, RenderParticleBoundsSnapshot, RenderParticlePreviousSpriteSnapshot,
    RenderParticleSpriteSnapshot, RenderPhaseQueue, RenderPhaseQueueSummary,
    RenderPointLightSnapshot, RenderPostProcessEffectStackSettings, RenderRectLightSnapshot,
    RenderReflectionProbeSnapshot, RenderResolvedPostProcessSettings, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderSpotLightSnapshot, RenderSpriteSnapshot,
    RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract, SceneViewportExtractRequest,
    SpriteExtract, SpritePhaseInput, ViewportCameraSnapshot, VolumeEvaluationError,
    VolumeEvaluationRequest, VolumeEvaluator, DEFAULT_CAMERA_EXPOSURE_EV100,
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
    pub scene_camera_entity: Option<EntityId>,
    pub scene_camera_order_report: Option<RenderCameraOrderReport>,
    pub core_pipeline: CorePipelineKind,
    pub anti_alias: AntiAliasSettings,
    pub target_size: Option<UVec2>,
}

impl RenderViewExtract {
    pub fn from_camera(camera: ViewportCameraSnapshot) -> Self {
        let core_pipeline = camera.core_pipeline_kind();
        let anti_alias = AntiAliasSettings::from_camera_msaa_samples(camera.msaa_samples);
        let target_size = camera_target_size(&camera);
        Self {
            camera,
            scene_camera_entity: None,
            scene_camera_order_report: None,
            core_pipeline,
            anti_alias,
            target_size,
        }
    }

    pub fn with_scene_camera_order_report(
        mut self,
        scene_camera_entity: EntityId,
        camera_order_report: RenderCameraOrderReport,
    ) -> Self {
        self.scene_camera_entity = Some(scene_camera_entity);
        self.scene_camera_order_report = Some(camera_order_report);
        self
    }

    pub fn apply_target_size(&mut self, target_size: UVec2) {
        self.target_size = Some(target_size);
        self.camera.apply_viewport_size(target_size);
    }

    pub fn effective_view_size(&self) -> UVec2 {
        let target_size = self
            .target_size
            .or_else(|| camera_target_size(&self.camera))
            .unwrap_or_else(|| UVec2::new(1, 1));
        self.camera.effective_viewport_size(target_size)
    }

    pub fn effective_render_size(&self) -> UVec2 {
        let target_size = self
            .target_size
            .or_else(|| camera_target_size(&self.camera))
            .unwrap_or_else(|| UVec2::new(1, 1));
        self.camera.effective_render_size(target_size)
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
    pub depth_bias: f32,
    pub render_queue: i32,
    pub material_queue: i32,
    pub order_in_layer: i32,
    pub ui_z_index: i32,
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
            depth_bias: 0.0,
            render_queue: 0,
            material_queue: 0,
            order_in_layer: 0,
            ui_z_index: 0,
        }
    }

    pub const fn with_depth_bias(mut self, depth_bias: f32) -> Self {
        self.depth_bias = depth_bias;
        self
    }

    pub const fn with_render_queue(mut self, render_queue: i32) -> Self {
        self.render_queue = render_queue;
        self
    }

    pub const fn with_material_queue(mut self, material_queue: i32) -> Self {
        self.material_queue = material_queue;
        self
    }

    pub const fn with_order_in_layer(mut self, order_in_layer: i32) -> Self {
        self.order_in_layer = order_in_layer;
        self
    }

    pub const fn with_ui_z_index(mut self, ui_z_index: i32) -> Self {
        self.ui_z_index = ui_z_index;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct GeometryExtract {
    pub meshes: Vec<RenderMeshSnapshot>,
    pub phase_inputs: Vec<GeometryPhaseInput>,
    pub phase_queue: RenderPhaseQueue,
    pub static_batches: Vec<StaticMeshBatchExtract>,
    pub virtual_geometry: Option<RenderVirtualGeometryExtract>,
    pub virtual_geometry_debug: Option<RenderVirtualGeometryDebugState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticMeshBatchExtract {
    pub model: ResourceHandle<ModelMarker>,
    pub mesh: Option<ResourceHandle<MeshMarker>>,
    pub material: ResourceHandle<MaterialMarker>,
    pub render_layer_mask: u32,
    pub mesh_indices: Vec<usize>,
    pub entities: Vec<EntityId>,
}

impl StaticMeshBatchExtract {
    pub fn instance_count(&self) -> usize {
        self.mesh_indices.len()
    }
}

impl GeometryExtract {
    pub fn from_meshes(core_pipeline: CorePipelineKind, meshes: Vec<RenderMeshSnapshot>) -> Self {
        let phase_inputs = meshes
            .iter()
            .enumerate()
            .map(|(mesh_index, mesh)| {
                GeometryPhaseInput::new(
                    mesh.node_id,
                    mesh_index,
                    RenderMaterialAlphaMode::Opaque,
                    mesh.transform.translation.z,
                )
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
                depth_bias: input.depth_bias,
                render_queue: input.render_queue,
                material_queue: input.material_queue,
                order_in_layer: input.order_in_layer,
                ui_z_index: input.ui_z_index,
            }),
        );

        let static_batches = build_static_mesh_batches(&meshes);

        Self {
            meshes,
            phase_inputs,
            phase_queue,
            static_batches,
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
                depth_bias: input.depth_bias,
                render_queue: input.render_queue,
                material_queue: input.material_queue,
                order_in_layer: input.order_in_layer,
                ui_z_index: input.ui_z_index,
            }),
        );
    }

    /// Builds a diagnostics summary from the current sorted mesh phase queue.
    pub fn phase_queue_summary(&self) -> RenderPhaseQueueSummary {
        self.phase_queue.summary()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct StaticMeshBatchKey {
    model: ResourceId,
    mesh: Option<ResourceId>,
    material: ResourceId,
    render_layer_mask: u32,
}

fn build_static_mesh_batches(meshes: &[RenderMeshSnapshot]) -> Vec<StaticMeshBatchExtract> {
    let mut batch_indices_by_key: BTreeMap<StaticMeshBatchKey, Vec<usize>> = BTreeMap::new();
    for (mesh_index, mesh) in meshes.iter().enumerate() {
        if mesh.mobility != Mobility::Static {
            continue;
        }
        batch_indices_by_key
            .entry(StaticMeshBatchKey {
                model: mesh.model.id(),
                mesh: mesh.mesh.map(ResourceHandle::id),
                material: mesh.material.id(),
                render_layer_mask: mesh.render_layer_mask,
            })
            .or_default()
            .push(mesh_index);
    }

    batch_indices_by_key
        .into_values()
        .filter(|mesh_indices| mesh_indices.len() > 1)
        .map(|mesh_indices| {
            let first_mesh = &meshes[mesh_indices[0]];
            StaticMeshBatchExtract {
                model: first_mesh.model,
                mesh: first_mesh.mesh,
                material: first_mesh.material,
                render_layer_mask: first_mesh.render_layer_mask,
                entities: mesh_indices
                    .iter()
                    .map(|mesh_index| meshes[*mesh_index].node_id)
                    .collect(),
                mesh_indices,
            }
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpritePhaseExtractInput {
    pub entity: EntityId,
    pub sprite_index: usize,
    pub material_alpha_mode: RenderMaterialAlphaMode,
    pub z_order: i32,
    pub depth: f32,
    pub depth_bias: f32,
    pub render_queue: i32,
    pub material_queue: i32,
    pub ui_z_index: i32,
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
            depth_bias: 0.0,
            render_queue: 0,
            material_queue: 0,
            ui_z_index: 0,
        }
    }

    pub const fn with_depth_bias(mut self, depth_bias: f32) -> Self {
        self.depth_bias = depth_bias;
        self
    }

    pub const fn with_render_queue(mut self, render_queue: i32) -> Self {
        self.render_queue = render_queue;
        self
    }

    pub const fn with_material_queue(mut self, material_queue: i32) -> Self {
        self.material_queue = material_queue;
        self
    }

    pub const fn with_ui_z_index(mut self, ui_z_index: i32) -> Self {
        self.ui_z_index = ui_z_index;
        self
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
                depth_bias: input.depth_bias,
                render_queue: input.render_queue,
                material_queue: input.material_queue,
                ui_z_index: input.ui_z_index,
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
    pub exposure: RenderExposureSettings,
    pub color_grading: RenderColorGradingSettings,
    pub effect_stack: RenderPostProcessEffectStackSettings,
    pub volumes: Vec<PostProcessVolumeExtract>,
    pub stack: PostProcessStackDescriptor,
    pub graph: PostProcessPassGraph,
}

impl Default for PostProcessExtract {
    fn default() -> Self {
        let bloom = RenderBloomSettings::default();
        let color_grading = RenderColorGradingSettings::default();
        Self::from_parts_with_effect_stack(
            PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: crate::core::math::Vec4::ZERO,
            },
            DisplayMode::Shaded,
            bloom,
            color_grading,
            RenderPostProcessEffectStackSettings::default(),
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
        temporal_history_enabled: bool,
        history_available: bool,
    ) -> Self {
        Self::from_parts_with_effect_stack(
            preview,
            display_mode,
            bloom,
            color_grading,
            RenderPostProcessEffectStackSettings::default(),
            temporal_history_enabled,
            history_available,
        )
    }

    pub fn from_parts_with_effect_stack(
        preview: PreviewEnvironmentExtract,
        display_mode: DisplayMode,
        bloom: RenderBloomSettings,
        color_grading: RenderColorGradingSettings,
        effect_stack: RenderPostProcessEffectStackSettings,
        temporal_history_enabled: bool,
        history_available: bool,
    ) -> Self {
        let stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_and_anti_alias(
                &bloom,
                &color_grading,
                RenderExposureSettings::manual_ev100(DEFAULT_CAMERA_EXPOSURE_EV100),
                &effect_stack,
                temporal_history_enabled,
                history_available,
                &AntiAliasSettings::off(),
            );
        let graph = stack.validated_graph();
        Self {
            preview,
            display_mode,
            bloom,
            exposure: RenderExposureSettings::manual_ev100(DEFAULT_CAMERA_EXPOSURE_EV100),
            color_grading,
            effect_stack,
            volumes: Vec::new(),
            stack,
            graph,
        }
    }

    pub fn rebuild_graph(&mut self, temporal_history_enabled: bool, history_available: bool) {
        self.stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_and_anti_alias(
                &self.bloom,
                &self.color_grading,
                self.exposure,
                &self.effect_stack,
                temporal_history_enabled,
                history_available,
                &AntiAliasSettings::off(),
            );
        self.graph = self.stack.validated_graph();
    }

    pub fn rebuild_graph_with_anti_alias(
        &mut self,
        temporal_history_enabled: bool,
        history_available: bool,
        anti_alias: &AntiAliasSettings,
    ) {
        self.stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_and_anti_alias(
                &self.bloom,
                &self.color_grading,
                self.exposure,
                &self.effect_stack,
                temporal_history_enabled,
                history_available,
                anti_alias,
            );
        self.graph = self.stack.validated_graph();
    }

    pub fn resolved_settings_for_camera(
        &self,
        camera_position: crate::core::math::Vec3,
        camera_volume_mask: &RenderLayerSet,
    ) -> Result<RenderResolvedPostProcessSettings, VolumeEvaluationError> {
        VolumeEvaluator::default().evaluate(VolumeEvaluationRequest {
            camera_position,
            camera_volume_mask,
            base_bloom: self.bloom,
            base_exposure: self.exposure,
            base_color_grading: self.color_grading,
            base_effect_stack: self.effect_stack,
            volumes: &self.volumes,
        })
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
    pub previous_sprites: Vec<RenderParticlePreviousSpriteSnapshot>,
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
    /// Builds a frame DTO from the legacy viewport packet for preview,
    /// roundtrip, and synthetic validation paths. Scene production producers
    /// should fill `RenderFrameExtract` directly because this adapter cannot
    /// recover advanced sidebands such as sprites, particles, VG payloads, or
    /// level-owned animation poses from a `SceneViewportRenderPacket`.
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
            post_process: {
                let mut post_process = PostProcessExtract::from_parts(
                    snapshot.preview.clone(),
                    snapshot.overlays.display_mode,
                    RenderBloomSettings::default(),
                    RenderColorGradingSettings::default(),
                    false,
                    false,
                );
                post_process.exposure =
                    RenderExposureSettings::manual_ev100(snapshot.scene.camera.exposure_ev100);
                post_process
            },
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
        self.view.apply_target_size(viewport_size);
    }

    pub fn with_viewport_size(mut self, viewport_size: UVec2) -> Self {
        self.apply_viewport_size(viewport_size);
        self
    }

    /// Builds a diagnostics summary for the frame's mesh and sprite phase queues.
    pub fn phase_queue_summary(&self) -> RenderFramePhaseQueueSummary {
        RenderFramePhaseQueueSummary::new(
            self.geometry.phase_queue_summary(),
            self.sprites.phase_queue_summary(),
        )
    }
}

fn camera_target_size(camera: &ViewportCameraSnapshot) -> Option<UVec2> {
    if let Some(viewport) = camera.viewport {
        return Some(viewport.physical_size);
    }
    match &camera.target {
        RenderCameraTarget::Headless { size } => Some(*size),
        RenderCameraTarget::PrimarySurface | RenderCameraTarget::Texture(_) => None,
    }
}
