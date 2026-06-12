use std::collections::{HashMap, HashSet};

use crate::core::framework::render::{
    build_mesh_phase_queue, render_mesh_stable_instance_key, GeometryPhaseInput, MeshPhaseInput,
    PrimitiveRelevance, RenderMeshSnapshot, RenderPhaseMeshSource, RenderPhaseQueue,
};
use crate::core::framework::scene::EntityId;
use crate::core::math::RenderVec4;
use crate::graphics::scene::gpu_scene::{
    GpuInstanceData, GpuPrimitiveData, GpuScene, GpuSceneEntry, GpuSceneUploadReport,
    GPU_PRIMITIVE_FLAG_CAST_SHADOWS, GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM,
    GPU_PRIMITIVE_FLAG_VISIBLE, GPU_SCENE_INVALID_PAYLOAD_SLOT,
};
use crate::graphics::scene::resources::{GpuMeshResource, ResourceStreamer};
use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::primitives::render_vec4_or;
use super::super::super::mesh_draw::VirtualGeometrySubmissionDetail;
use super::super::super::mesh_draw::{MeshDraw, MeshDrawGeometrySource};
use super::super::create_mesh_draw::create_mesh_draw;
use super::super::indexed_indirect_args::IndexedIndirectArgs;
use super::build_mesh_draw_build_context::build_mesh_draw_build_context;
use super::extend_pending_draws_for_mesh_instance::extend_pending_draws_for_mesh_instance;
use super::pending_mesh_draw::{PendingMeshGeometry, PendingSkinnedGpuSource};

pub(crate) struct BuiltMeshDraws {
    draws: Vec<MeshDraw>,
    gpu_scene_upload_report: GpuSceneUploadReport,
    indirect_segment_count: u32,
    indirect_args_count: u32,
    indirect_args_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_submission_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_authority_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_draw_ref_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_segment_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
}

impl BuiltMeshDraws {
    pub(crate) fn into_draws(self) -> Vec<MeshDraw> {
        self.draws
    }

    pub(crate) fn gpu_scene_upload_report(&self) -> GpuSceneUploadReport {
        self.gpu_scene_upload_report
    }

    pub(crate) fn indirect_segment_count(&self) -> u32 {
        self.indirect_segment_count
    }

    pub(crate) fn indirect_args_count(&self) -> u32 {
        self.indirect_args_count
    }

    pub(crate) fn indirect_args_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_args_buffer.clone()
    }

    pub(crate) fn indirect_submission_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_submission_buffer.clone()
    }

    pub(crate) fn indirect_authority_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_authority_buffer.clone()
    }

    pub(crate) fn indirect_draw_ref_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_draw_ref_buffer.clone()
    }

    pub(crate) fn indirect_segment_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_segment_buffer.clone()
    }
}

pub(crate) fn build_mesh_draws(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    _encoder: &mut wgpu::CommandEncoder,
    material_texture_layout: &wgpu::BindGroupLayout,
    gpu_scene: &mut GpuScene,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    virtual_geometry_enabled: bool,
) -> BuiltMeshDraws {
    let build_context = build_mesh_draw_build_context(frame, virtual_geometry_enabled);
    let mut pending_draws = Vec::new();
    for mesh_instance in phase_ordered_meshes(frame, streamer) {
        extend_pending_draws_for_mesh_instance(
            &mut pending_draws,
            streamer,
            frame,
            &build_context,
            mesh_instance,
        );
    }
    let (gpu_scene_upload_report, gpu_scene_entries) =
        sync_gpu_scene_pending_draws(device, queue, gpu_scene, &pending_draws);
    let visibility_states = mesh_visibility_states(frame);
    let indirect_segment_count = 0;
    let indirect_args_count = 0;
    let indirect_draw_ref_buffer = None;
    let indirect_submission_buffer = None;
    let indirect_authority_buffer = None;
    let indirect_segment_buffer = None;
    let indirect_args_offsets = Vec::new();
    let pending_draw_draw_ref_indices = Vec::new();
    let pending_draw_submission_tokens = Vec::new();
    let pending_draw_submission_details = Vec::new();
    let shared_indirect_args_buffer = None;
    let indirect_args_buffer = None;
    let indirect_args_stride = std::mem::size_of::<IndexedIndirectArgs>() as u64;

    let pending_draws = pending_draws.into_iter().map(Some).collect::<Vec<_>>();
    let mut ordered_pending_draws = Vec::new();
    ordered_pending_draws.extend(pending_draws.into_iter().enumerate().filter_map(
        |(index, pending_draw)| {
            let pending_draw = pending_draw?;
            Some((
                indirect_args_offsets
                    .get(index)
                    .copied()
                    .unwrap_or((index as u64) * indirect_args_stride),
                index,
                pending_draw_submission_details
                    .get(index)
                    .copied()
                    .flatten()
                    .or_else(|| {
                        submission_detail_from_draw_ref(
                            pending_draw.indirect_draw_ref,
                            pending_draw_submission_tokens.get(index).copied(),
                            pending_draw_draw_ref_indices.get(index).copied(),
                            indirect_args_offsets.get(index).copied(),
                            indirect_args_stride,
                        )
                    }),
                pending_draw,
            ))
        },
    ));
    BuiltMeshDraws {
        draws: ordered_pending_draws
            .into_iter()
            .map(
                |(indirect_args_offset, original_index, submission_detail, pending_draw)| {
                    let gpu_scene_instance_span = gpu_scene_entries
                        .get(&render_mesh_stable_instance_key(
                            pending_draw.source_entity,
                            pending_draw.source_draw_ordinal,
                        ))
                        .map(|entry| (entry.first_instance_index, entry.instance_count))
                        .expect("pending mesh draw must have a synchronized GPUScene entry");
                    let supports_skinned_gpu_skinning =
                        pending_draw.pipeline_key.uses_fallback_shader();
                    let skinned_gpu_source = pending_draw.skinned_gpu_source;
                    let resolved_skinned_gpu_source = supports_skinned_gpu_skinning
                        .then(|| {
                            skinned_gpu_source
                                .as_ref()
                                .map(|source| resolve_skinned_gpu_source(device, source))
                        })
                        .flatten();
                    let (
                        mesh,
                        geometry_source,
                        resolved_skinned_gpu_source,
                        skinned_gpu_source_uses_cpu_morphed_source,
                        skinned_gpu_skinning_enabled,
                    ) = match pending_draw.mesh {
                        PendingMeshGeometry::Prepared(mesh) => {
                            (mesh, MeshDrawGeometrySource::Prepared, None, false, false)
                        }
                        PendingMeshGeometry::Dynamic(primitive) => {
                            if let Some((mesh, geometry_source, uses_cpu_morphed_source)) =
                                resolved_skinned_gpu_source
                            {
                                (
                                    mesh.clone(),
                                    geometry_source,
                                    Some(mesh),
                                    uses_cpu_morphed_source,
                                    true,
                                )
                            } else {
                                (
                                    std::sync::Arc::new(GpuMeshResource::from_asset(
                                        device, primitive,
                                    )),
                                    MeshDrawGeometrySource::Dynamic,
                                    None,
                                    false,
                                    false,
                                )
                            }
                        }
                    };
                    let previous_skinned_joint_palette = if skinned_gpu_skinning_enabled {
                        pending_draw.previous_skinned_joint_palette
                    } else {
                        None
                    };
                    let has_previous_motion_vector_transform = pending_draw
                        .has_previous_motion_vector_transform
                        && (!skinned_gpu_skinning_enabled
                            || previous_skinned_joint_palette.is_some());
                    let mut mesh_draw = create_mesh_draw(
                        device,
                        gpu_scene,
                        material_texture_layout,
                        mesh,
                        geometry_source,
                        pending_draw.mobility,
                        pending_draw.source_entity,
                        pending_draw.source_draw_ordinal,
                        pending_draw.static_state,
                        pending_draw.material_textures,
                        pending_draw.material_uniform,
                        pending_draw.standard_material_uniform,
                        pending_draw.pipeline_key,
                        pending_draw.cast_shadows,
                        has_previous_motion_vector_transform,
                        pending_draw.mesh_lod,
                        pending_draw.skinned,
                        pending_draw.skinned_joint_palette,
                        previous_skinned_joint_palette,
                        resolved_skinned_gpu_source,
                        skinned_gpu_source_uses_cpu_morphed_source,
                        skinned_gpu_skinning_enabled,
                        pending_draw.first_index,
                        pending_draw.draw_index_count,
                        shared_indirect_args_buffer.clone(),
                        indirect_args_offset,
                        submission_detail.or_else(|| {
                            submission_detail_from_draw_ref(
                                pending_draw.indirect_draw_ref,
                                pending_draw_submission_tokens.get(original_index).copied(),
                                pending_draw_draw_ref_indices.get(original_index).copied(),
                                Some(indirect_args_offset),
                                indirect_args_stride,
                            )
                        }),
                    )
                    .with_gpu_scene_instance_span(
                        gpu_scene_instance_span.0,
                        gpu_scene_instance_span.1,
                    );
                    if let Some(visibility) =
                        visibility_states.get(&pending_draw.source_entity).copied()
                    {
                        mesh_draw = mesh_draw.with_visibility(
                            visibility.relevance,
                            visibility.main_view_visible,
                            visibility.shadow_view_visible,
                        );
                    }
                    mesh_draw
                },
            )
            .collect(),
        gpu_scene_upload_report,
        indirect_segment_count,
        indirect_args_count,
        indirect_args_buffer,
        indirect_submission_buffer,
        indirect_authority_buffer,
        indirect_draw_ref_buffer,
        indirect_segment_buffer,
    }
}

#[derive(Clone, Copy)]
struct MeshVisibilityState {
    relevance: PrimitiveRelevance,
    main_view_visible: bool,
    shadow_view_visible: bool,
}

fn mesh_visibility_states(frame: &ViewportRenderFrame) -> HashMap<EntityId, MeshVisibilityState> {
    let Some(frame_visibility) = frame.frame_visibility() else {
        return HashMap::new();
    };
    let main_visible_indices = frame_visibility
        .main_view()
        .map(|view| view.visible.iter().copied().collect::<HashSet<_>>())
        .unwrap_or_default();
    let shadow_views = frame_visibility.shadow_views().collect::<Vec<_>>();
    let shadow_visible_indices = shadow_views
        .iter()
        .flat_map(|view| view.visible.iter().copied())
        .collect::<HashSet<_>>();
    let has_shadow_views = !shadow_views.is_empty();

    frame_visibility
        .entities
        .iter()
        .enumerate()
        .map(|(index, entity)| {
            let index =
                u32::try_from(index).expect("frame visibility primitive index exceeds u32 range");
            let relevance = frame_visibility
                .relevance
                .get(index as usize)
                .copied()
                .unwrap_or_default();
            (
                *entity,
                MeshVisibilityState {
                    relevance,
                    main_view_visible: main_visible_indices.contains(&index),
                    shadow_view_visible: if has_shadow_views {
                        shadow_visible_indices.contains(&index)
                    } else {
                        relevance.shadow_caster()
                    },
                },
            )
        })
        .collect()
}

fn sync_gpu_scene_pending_draws(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    gpu_scene: &mut GpuScene,
    pending_draws: &[super::pending_mesh_draw::PendingMeshDraw],
) -> (GpuSceneUploadReport, HashMap<u64, GpuSceneEntry>) {
    let mut live_keys = HashSet::new();
    let mut entries = HashMap::new();
    for pending_draw in pending_draws {
        let stable_instance_key = render_mesh_stable_instance_key(
            pending_draw.source_entity,
            pending_draw.source_draw_ordinal,
        );
        live_keys.insert(stable_instance_key);
        let entry = gpu_scene.register(device, stable_instance_key, 1);
        gpu_scene.write_primitive(entry, primitive_data_for_pending_draw(pending_draw, entry));
        gpu_scene.write_instances(
            entry,
            &[instance_data_for_pending_draw(pending_draw, entry)],
        );
        gpu_scene.set_transform_revision(stable_instance_key, pending_draw.transform_revision);
        entries.insert(stable_instance_key, entry);
    }
    gpu_scene.retain_registered_keys(&live_keys);
    (gpu_scene.flush_updates(queue), entries)
}

fn primitive_data_for_pending_draw(
    pending_draw: &super::pending_mesh_draw::PendingMeshDraw,
    entry: GpuSceneEntry,
) -> GpuPrimitiveData {
    let mut flags = GPU_PRIMITIVE_FLAG_VISIBLE;
    if pending_draw.cast_shadows {
        flags |= GPU_PRIMITIVE_FLAG_CAST_SHADOWS;
    }
    if pending_draw.has_previous_motion_vector_transform {
        flags |= GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM;
    }

    GpuPrimitiveData {
        bounds_center: [
            pending_draw.model_matrix[3][0],
            pending_draw.model_matrix[3][1],
            pending_draw.model_matrix[3][2],
        ],
        bounds_radius: approximate_transform_radius(&pending_draw.model_matrix),
        tint: render_vec4_or(pending_draw.draw_tint, RenderVec4::ONE).to_array(),
        shadow_params: shadow_params_from_pending_draw(pending_draw),
        motion_params: motion_params_from_pending_draw(pending_draw),
        flags,
        first_instance_index: entry.first_instance_index,
        instance_count: entry.instance_count,
        payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
    }
}

fn instance_data_for_pending_draw(
    pending_draw: &super::pending_mesh_draw::PendingMeshDraw,
    entry: GpuSceneEntry,
) -> GpuInstanceData {
    GpuInstanceData {
        world_from_local: pending_draw.model_matrix,
        prev_world_from_local: pending_draw.previous_model_matrix,
        primitive_index: entry.primitive_index,
        flags: 0,
        payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
        _pad0: 0,
    }
}

fn shadow_params_from_pending_draw(
    pending_draw: &super::pending_mesh_draw::PendingMeshDraw,
) -> [f32; 4] {
    let alpha_cutoff = pending_draw
        .pipeline_key
        .alpha_cutoff_bits
        .map(f32::from_bits)
        .filter(|cutoff| cutoff.is_finite())
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);

    [
        if pending_draw.pipeline_key.is_alpha_mask() {
            1.0
        } else {
            0.0
        },
        alpha_cutoff,
        if pending_draw.receive_shadows {
            1.0
        } else {
            0.0
        },
        0.0,
    ]
}

fn motion_params_from_pending_draw(
    pending_draw: &super::pending_mesh_draw::PendingMeshDraw,
) -> [f32; 4] {
    [
        if pending_draw.has_previous_motion_vector_transform {
            1.0
        } else {
            0.0
        },
        if pending_draw.skinned { 1.0 } else { 0.0 },
        if pending_draw.skinned
            && pending_draw.has_previous_motion_vector_transform
            && pending_draw.previous_skinned_joint_palette.is_some()
        {
            1.0
        } else {
            0.0
        },
        if pending_draw.pipeline_key.has_normal_texture {
            1.0
        } else {
            0.0
        },
    ]
}

fn approximate_transform_radius(model_matrix: &[[f32; 4]; 4]) -> f32 {
    let x = column_length(model_matrix[0]);
    let y = column_length(model_matrix[1]);
    let z = column_length(model_matrix[2]);
    x.max(y).max(z)
}

fn column_length(column: [f32; 4]) -> f32 {
    (column[0] * column[0] + column[1] * column[1] + column[2] * column[2]).sqrt()
}

fn resolve_skinned_gpu_source(
    device: &wgpu::Device,
    source: &PendingSkinnedGpuSource,
) -> (
    std::sync::Arc<GpuMeshResource>,
    MeshDrawGeometrySource,
    bool,
) {
    match source {
        PendingSkinnedGpuSource::Prepared(mesh) => {
            (mesh.clone(), MeshDrawGeometrySource::Prepared, false)
        }
        PendingSkinnedGpuSource::CpuMorphed(primitive) => (
            std::sync::Arc::new(GpuMeshResource::from_asset(device, primitive.clone())),
            MeshDrawGeometrySource::DynamicGpuSkinningSource,
            source.uses_cpu_morphed_source(),
        ),
    }
}

fn phase_ordered_meshes<'a>(
    frame: &'a ViewportRenderFrame,
    streamer: &ResourceStreamer,
) -> Vec<&'a RenderMeshSnapshot> {
    phase_ordered_meshes_with_material_offsets(frame, |mesh| material_sort_offsets(streamer, mesh))
}

fn phase_ordered_meshes_with_material_offsets<'a>(
    frame: &'a ViewportRenderFrame,
    material_sort_offsets: impl Fn(&RenderMeshSnapshot) -> MaterialPhaseSortOffsets,
) -> Vec<&'a RenderMeshSnapshot> {
    let phase_queue = &frame.extract.geometry.phase_queue;
    if phase_queue.items.is_empty() {
        return frame.meshes().iter().collect();
    }

    let material_adjusted_phase_queue = material_adjusted_phase_queue(frame, material_sort_offsets)
        .unwrap_or_else(|| frame.extract.geometry.phase_queue.clone());
    meshes_from_phase_queue(frame, &material_adjusted_phase_queue)
}

fn meshes_from_phase_queue<'a>(
    frame: &'a ViewportRenderFrame,
    phase_queue: &RenderPhaseQueue,
) -> Vec<&'a RenderMeshSnapshot> {
    phase_queue
        .items
        .iter()
        .filter_map(|item| match item.mesh_source {
            RenderPhaseMeshSource::MeshIndex(index) => frame.meshes().get(index),
            RenderPhaseMeshSource::SpriteIndex(_) => None,
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct MaterialPhaseSortOffsets {
    render_queue: i32,
    material_queue: i32,
    depth_bias: f32,
}

fn material_sort_offsets(
    streamer: &ResourceStreamer,
    mesh: &RenderMeshSnapshot,
) -> MaterialPhaseSortOffsets {
    streamer
        .material(&mesh.material.id())
        .map(|material| MaterialPhaseSortOffsets {
            render_queue: material.render_queue,
            material_queue: material.material_queue,
            depth_bias: material.depth_bias,
        })
        .unwrap_or_default()
}

fn material_adjusted_phase_queue(
    frame: &ViewportRenderFrame,
    material_sort_offsets: impl Fn(&RenderMeshSnapshot) -> MaterialPhaseSortOffsets,
) -> Option<RenderPhaseQueue> {
    let phase_inputs = frame.extract.geometry.phase_inputs.as_slice();
    (!phase_inputs.is_empty()).then(|| {
        build_mesh_phase_queue(
            frame.extract.view.core_pipeline,
            phase_inputs.iter().map(|input| {
                let offsets = frame
                    .meshes()
                    .get(input.mesh_index)
                    .map(|mesh| material_sort_offsets(mesh))
                    .unwrap_or_default();
                mesh_phase_input_with_material_offsets(input, offsets)
            }),
        )
    })
}

fn mesh_phase_input_with_material_offsets<'a>(
    input: &'a GeometryPhaseInput,
    offsets: MaterialPhaseSortOffsets,
) -> MeshPhaseInput<'a> {
    MeshPhaseInput {
        entity: input.entity,
        mesh_index: input.mesh_index,
        material_alpha_mode: &input.material_alpha_mode,
        depth: input.depth,
        depth_bias: input.depth_bias + offsets.depth_bias,
        render_queue: input.render_queue.saturating_add(offsets.render_queue),
        material_queue: input.material_queue.saturating_add(offsets.material_queue),
        order_in_layer: input.order_in_layer,
        ui_z_index: input.ui_z_index,
    }
}

fn submission_detail_from_draw_ref(
    draw_ref: Option<super::pending_mesh_draw::VirtualGeometryIndirectDrawRef>,
    submission_token: Option<u32>,
    draw_ref_index: Option<u32>,
    indirect_args_offset: Option<u64>,
    indirect_args_stride: u64,
) -> Option<VirtualGeometrySubmissionDetail> {
    let draw_ref = draw_ref?;
    let submission_token = submission_token.unwrap_or(u32::MAX);
    Some(VirtualGeometrySubmissionDetail::new(
        draw_ref.segment_key.instance_index,
        draw_ref.segment_key.entity,
        draw_ref.segment_key.page_id,
        if submission_token == u32::MAX {
            0
        } else {
            submission_token >> 16
        },
        if submission_token == u32::MAX {
            0
        } else {
            submission_token & 0xffff
        },
        draw_ref_index.unwrap_or_else(|| {
            indirect_args_offset
                .map(|offset| (offset / indirect_args_stride) as u32)
                .unwrap_or_default()
        }),
        draw_ref.segment_key.cluster_start_ordinal,
        draw_ref.segment_key.cluster_span_count,
        draw_ref.segment_key.cluster_total_count,
        draw_ref.segment_key.submission_slot,
        match draw_ref.segment_key.state {
            0 => crate::core::framework::render::RenderVirtualGeometryExecutionState::Resident,
            1 => crate::core::framework::render::RenderVirtualGeometryExecutionState::PendingUpload,
            _ => crate::core::framework::render::RenderVirtualGeometryExecutionState::Missing,
        },
        draw_ref.segment_key.lineage_depth,
        draw_ref.segment_key.lod_level,
        draw_ref.segment_key.frontier_rank,
    ))
}

#[cfg(test)]
mod tests {
    use super::{phase_ordered_meshes_with_material_offsets, MaterialPhaseSortOffsets};
    use crate::core::framework::render::{
        FallbackSkyboxKind, GeometryExtract, GeometryPhaseInput, PreviewEnvironmentExtract,
        RenderFrameExtract, RenderMaterialAlphaMode, RenderMeshSnapshot, RenderOverlayExtract,
        RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
        ViewportCameraSnapshot,
    };
    use crate::core::framework::scene::Mobility;
    use crate::core::math::{Transform, UVec2, Vec4};
    use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
    use crate::graphics::ViewportRenderFrame;

    #[test]
    fn phase_ordered_meshes_follow_extract_phase_queue_instead_of_mesh_vector_order() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(9),
            RenderSceneSnapshot {
                scene: RenderSceneGeometryExtract {
                    camera: ViewportCameraSnapshot::default(),
                    meshes: vec![test_mesh(30), test_mesh(10), test_mesh(20)],
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
                    clear_color: Vec4::ZERO,
                },
                virtual_geometry_debug: None,
            },
        );
        extract.geometry = GeometryExtract::from_meshes_and_phase_inputs(
            extract.view.core_pipeline,
            extract.geometry.meshes.clone(),
            vec![
                GeometryPhaseInput::new(30, 0, RenderMaterialAlphaMode::Blend, 3.0),
                GeometryPhaseInput::new(10, 1, RenderMaterialAlphaMode::Opaque, 1.0),
                GeometryPhaseInput::new(20, 2, RenderMaterialAlphaMode::Mask { cutoff: 0.5 }, 2.0),
            ],
        );
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(320, 240));

        assert_eq!(
            phase_ordered_meshes_with_material_offsets(&frame, |_| {
                MaterialPhaseSortOffsets::default()
            })
            .into_iter()
            .map(|mesh| mesh.node_id)
            .collect::<Vec<_>>(),
            vec![10, 20, 30]
        );
    }

    #[test]
    fn phase_ordered_meshes_apply_material_sort_offsets_to_extract_phase_queue() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(10),
            RenderSceneSnapshot {
                scene: RenderSceneGeometryExtract {
                    camera: ViewportCameraSnapshot::default(),
                    meshes: vec![test_mesh(10), test_mesh(20), test_mesh(30)],
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
                    clear_color: Vec4::ZERO,
                },
                virtual_geometry_debug: None,
            },
        );
        extract.geometry = GeometryExtract::from_meshes_and_phase_inputs(
            extract.view.core_pipeline,
            extract.geometry.meshes.clone(),
            vec![
                GeometryPhaseInput::new(10, 0, RenderMaterialAlphaMode::Opaque, 1.0),
                GeometryPhaseInput::new(20, 1, RenderMaterialAlphaMode::Opaque, 2.0),
                GeometryPhaseInput::new(30, 2, RenderMaterialAlphaMode::Opaque, 3.0),
            ],
        );
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(320, 240));

        assert_eq!(
            phase_ordered_meshes_with_material_offsets(&frame, |mesh| match mesh.node_id {
                20 => MaterialPhaseSortOffsets {
                    render_queue: -5,
                    material_queue: 0,
                    depth_bias: 0.0,
                },
                30 => MaterialPhaseSortOffsets {
                    render_queue: 0,
                    material_queue: -3,
                    depth_bias: -2.5,
                },
                _ => MaterialPhaseSortOffsets::default(),
            })
            .into_iter()
            .map(|mesh| mesh.node_id)
            .collect::<Vec<_>>(),
            vec![20, 30, 10]
        );
    }

    fn test_mesh(node_id: u64) -> RenderMeshSnapshot {
        RenderMeshSnapshot {
            node_id,
            stable_instance_key: node_id << 16,
            transform_revision: 0,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(&format!(
                "builtin://test-model/{node_id}"
            ))),
            mesh: None,
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                &format!("builtin://test-material/{node_id}"),
            )),
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            render_layer_mask: u32::MAX,
        }
    }
}
