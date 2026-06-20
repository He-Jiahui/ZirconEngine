use crate::core::math::UVec2;
use crate::core::resource::{MaterialMarker, MeshMarker, ModelMarker, ResourceHandle, ResourceId};
use std::collections::BTreeMap;

use crate::core::framework::animation::AnimationPoseOutput;
use crate::core::framework::scene::{EntityId, Mobility, WorldHandle};

mod particle_extract_policy;

use super::{
    build_mesh_phase_queue, build_sprite_phase_queue, AntiAliasSettings, CameraRenderDescriptor,
    CorePipelineKind, DisplayMode, FallbackSkyboxKind, MeshPhaseInput, PostProcessPassGraph,
    PostProcessStackDescriptor, PostProcessVolumeExtract, PreviewEnvironmentExtract,
    RenderAmbientLightSnapshot, RenderBakedLightingExtract, RenderBloomSettings,
    RenderCameraOrderReport, RenderCameraTarget, RenderColorGradingSettings,
    RenderDirectionalLightSnapshot, RenderExposureSettings, RenderFramePhaseQueueSummary,
    RenderHybridGiExtract, RenderLayerSet, RenderMaterialAlphaMode, RenderMeshSnapshot,
    RenderOverlayExtract, RenderParticleBoundsSnapshot, RenderParticlePreviousSpriteSnapshot,
    RenderParticleSpriteSnapshot, RenderPhaseQueue, RenderPhaseQueueSummary,
    RenderPointLightSnapshot, RenderPostProcessEffectStackSettings, RenderQueueValue,
    RenderRectLightSnapshot, RenderReflectionProbeSnapshot, RenderResolvedPostProcessSettings,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderSpotLightSnapshot, RenderSpriteSnapshot,
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
    pub cameras: Vec<CameraRenderDescriptor>,
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
        let descriptor = CameraRenderDescriptor::from_camera_payload(None, camera.clone());
        let target_size = camera_target_size_from_descriptor(Some(&descriptor));
        Self {
            cameras: vec![descriptor],
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

    pub fn with_cameras(mut self, cameras: Vec<CameraRenderDescriptor>) -> Self {
        self.cameras = cameras;
        self
    }

    pub fn with_selected_camera_descriptor(
        mut self,
        mut descriptor: CameraRenderDescriptor,
    ) -> Self {
        descriptor.apply_target_size(
            self.target_size
                .or_else(|| camera_target_size_from_descriptor(Some(&descriptor)))
                .unwrap_or_else(|| UVec2::new(1, 1)),
        );
        self.core_pipeline = descriptor.camera.core_pipeline_kind();
        self.anti_alias =
            AntiAliasSettings::from_camera_msaa_samples(descriptor.camera.msaa_samples);
        self.camera = descriptor.camera.clone();
        self.scene_camera_entity = descriptor.entity;
        self.cameras = vec![descriptor];
        self
    }

    pub fn selected_camera_descriptor(&self) -> Option<&CameraRenderDescriptor> {
        self.scene_camera_entity
            .and_then(|entity| {
                self.cameras
                    .iter()
                    .find(|camera| camera.entity == Some(entity))
            })
            .or_else(|| self.cameras.first())
    }

    pub fn selected_camera_descriptor_mut(&mut self) -> Option<&mut CameraRenderDescriptor> {
        if let Some(entity) = self.scene_camera_entity {
            if let Some(index) = self
                .cameras
                .iter()
                .position(|camera| camera.entity == Some(entity))
            {
                return self.cameras.get_mut(index);
            }
        }
        self.cameras.first_mut()
    }

    pub fn selected_camera_target(&self) -> &RenderCameraTarget {
        self.selected_camera_descriptor()
            .map(|camera| &camera.target)
            .expect("render view extract must carry a selected camera descriptor")
    }

    pub fn selected_camera_layers(&self) -> &RenderLayerSet {
        self.selected_camera_descriptor()
            .map(|camera| &camera.culling_mask)
            .expect("render view extract must carry a selected camera descriptor")
    }

    pub fn selected_effective_camera(&self) -> ViewportCameraSnapshot {
        self.selected_camera_descriptor()
            .map(CameraRenderDescriptor::as_effective_camera)
            .unwrap_or_else(|| self.camera.clone())
    }

    pub fn sync_selected_descriptor_camera_payload(&mut self) {
        let camera_payload = self.camera.clone();
        if let Some(camera) = self.selected_camera_descriptor_mut() {
            camera.camera = camera_payload;
            self.camera = camera.camera.clone();
        }
    }

    pub fn apply_target_size(&mut self, target_size: UVec2) {
        self.target_size = Some(target_size);
        self.sync_selected_descriptor_camera_payload();
        if let Some(camera) = self.selected_camera_descriptor_mut() {
            camera.apply_target_size(target_size);
            self.camera = camera.camera.clone();
        } else {
            self.camera.apply_viewport_size(target_size);
        }
    }

    pub fn effective_view_size(&self) -> UVec2 {
        let camera = self.selected_effective_camera();
        let target_size = self
            .target_size
            .or_else(|| camera_target_size_from_descriptor(self.selected_camera_descriptor()))
            .unwrap_or_else(|| UVec2::new(1, 1));
        self.selected_camera_descriptor()
            .map(|camera| camera.effective_viewport_size(target_size))
            .unwrap_or_else(|| camera.effective_viewport_size(target_size))
    }

    pub fn effective_render_size(&self) -> UVec2 {
        let camera = self.selected_effective_camera();
        let target_size = self
            .target_size
            .or_else(|| camera_target_size_from_descriptor(self.selected_camera_descriptor()))
            .unwrap_or_else(|| UVec2::new(1, 1));
        self.selected_camera_descriptor()
            .map(|camera| camera.effective_render_size(target_size))
            .unwrap_or_else(|| camera.effective_render_size(target_size))
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
                queue: resolved_phase_queue(
                    &input.material_alpha_mode,
                    input.render_queue,
                    input.material_queue,
                ),
                depth: input.depth,
                depth_bias: input.depth_bias,
                camera_order: 0,
                sorting_layer: 0,
                order_in_layer: input.order_in_layer,
                y_sort: None,
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
                queue: resolved_phase_queue(
                    &input.material_alpha_mode,
                    input.render_queue,
                    input.material_queue,
                ),
                depth: input.depth,
                depth_bias: input.depth_bias,
                camera_order: 0,
                sorting_layer: 0,
                order_in_layer: input.order_in_layer,
                y_sort: None,
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
                queue: resolved_phase_queue(
                    &input.material_alpha_mode,
                    input.render_queue,
                    input.material_queue,
                ),
                z_order: input.z_order,
                depth: input.depth,
                depth_bias: input.depth_bias,
                camera_order: 0,
                sorting_layer: 0,
                y_sort: None,
                ui_z_index: input.ui_z_index,
            }),
        );

        Self {
            sprites,
            phase_queue,
        }
    }
}

fn resolved_phase_queue(
    alpha_mode: &RenderMaterialAlphaMode,
    render_queue: i32,
    material_queue: i32,
) -> RenderQueueValue {
    RenderQueueValue::from_authored_queue(alpha_mode, render_queue)
        .with_material_offset_i32(material_queue)
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

    pub fn with_selected_camera_descriptor(mut self, descriptor: CameraRenderDescriptor) -> Self {
        self.view = self.view.with_selected_camera_descriptor(descriptor);
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

fn camera_target_size_from_descriptor(camera: Option<&CameraRenderDescriptor>) -> Option<UVec2> {
    let camera = camera?;
    if let Some(viewport) = camera.viewport_rect {
        return Some(viewport.physical_size);
    }
    match &camera.target {
        RenderCameraTarget::Headless { size } => Some(*size),
        RenderCameraTarget::PrimarySurface | RenderCameraTarget::Texture(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{RenderCameraTarget, RenderLayerSet, RenderViewportRect};
    use crate::core::resource::TextureMarker;

    #[test]
    fn render_view_apply_target_size_preserves_descriptor_target_and_layers() {
        let mut view = RenderViewExtract::from_camera(ViewportCameraSnapshot::default());
        let mut descriptor =
            CameraRenderDescriptor::from_camera_payload(Some(7), ViewportCameraSnapshot::default());
        descriptor.target = RenderCameraTarget::Headless {
            size: UVec2::new(320, 180),
        };
        descriptor.viewport_rect = Some(RenderViewportRect::new(UVec2::ZERO, UVec2::new(320, 160)));
        descriptor.culling_mask = RenderLayerSet::layer(3);
        descriptor.volume_mask = RenderLayerSet::layer(4);
        view.scene_camera_entity = Some(7);
        view.cameras = vec![descriptor];

        view.apply_target_size(UVec2::new(1280, 720));

        let selected = view
            .selected_camera_descriptor()
            .expect("selected scene camera descriptor should remain present");
        assert!(matches!(
            selected.target,
            RenderCameraTarget::Headless {
                size: UVec2 { x: 320, y: 180 }
            }
        ));
        assert_eq!(selected.culling_mask.to_legacy_mask_lossy(), 1 << 3);
        assert_eq!(selected.volume_mask.to_legacy_mask_lossy(), 1 << 4);
        assert!((view.camera.aspect_ratio - 2.0).abs() < 1.0e-4);
    }

    #[test]
    fn render_frame_extract_selected_camera_descriptor_replaces_active_selection_only() {
        let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
            "tests/camera-loop/rt",
        ));
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(10),
            RenderSceneSnapshot {
                scene: RenderSceneGeometryExtract {
                    camera: ViewportCameraSnapshot::default(),
                    meshes: Vec::new(),
                    directional_lights: Vec::new(),
                    point_lights: Vec::new(),
                    spot_lights: Vec::new(),
                    ambient_lights: Vec::new(),
                    rect_lights: Vec::new(),
                },
                overlays: RenderOverlayExtract::default(),
                preview: PreviewEnvironmentExtract {
                    lighting_enabled: false,
                    skybox_enabled: false,
                    fallback_skybox: FallbackSkyboxKind::None,
                    clear_color: crate::core::math::Vec4::ZERO,
                },
                virtual_geometry_debug: None,
            },
        );
        let mut primary =
            CameraRenderDescriptor::from_camera_payload(Some(1), ViewportCameraSnapshot::default());
        primary.render_order = 0;
        let mut target =
            CameraRenderDescriptor::from_camera_payload(Some(2), ViewportCameraSnapshot::default());
        target.render_order = 10;
        target.target = RenderCameraTarget::Texture(texture);
        target.culling_mask = RenderLayerSet::layer(4);
        extract.view = extract.view.with_cameras(vec![primary, target.clone()]);

        let selected = extract.with_selected_camera_descriptor(target.clone());

        assert_eq!(selected.view.scene_camera_entity, Some(2));
        assert_eq!(selected.view.cameras.len(), 1);
        assert_eq!(selected.view.selected_camera_target(), &target.target);
        assert_eq!(
            selected
                .view
                .selected_camera_layers()
                .to_legacy_mask_lossy(),
            1 << 4
        );
    }
}
