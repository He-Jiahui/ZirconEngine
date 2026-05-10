use crate::core::framework::render::{RenderPhaseMeshSource, RenderPhaseQueue};
use crate::graphics::scene::resources::GpuMeshResource;
use crate::graphics::types::ViewportRenderFrame;

use super::super::super::mesh_draw::MeshDraw;
use super::super::super::mesh_draw::VirtualGeometrySubmissionDetail;
use super::super::create_mesh_draw::create_mesh_draw;
use super::super::indexed_indirect_args::IndexedIndirectArgs;
use super::build_mesh_draw_build_context::build_mesh_draw_build_context;
use super::extend_pending_draws_for_mesh_instance::extend_pending_draws_for_mesh_instance;

pub(crate) struct BuiltMeshDraws {
    draws: Vec<MeshDraw>,
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
    _encoder: &mut wgpu::CommandEncoder,
    model_layout: &wgpu::BindGroupLayout,
    streamer: &crate::graphics::scene::resources::ResourceStreamer,
    frame: &ViewportRenderFrame,
    virtual_geometry_enabled: bool,
) -> BuiltMeshDraws {
    let build_context = build_mesh_draw_build_context(frame, virtual_geometry_enabled);
    let mut pending_draws = Vec::new();
    for mesh_instance in phase_ordered_meshes(frame) {
        extend_pending_draws_for_mesh_instance(
            &mut pending_draws,
            streamer,
            frame,
            &build_context,
            mesh_instance,
        );
    }
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
                    let mesh = match pending_draw.mesh {
                        super::pending_mesh_draw::PendingMeshGeometry::Prepared(mesh) => mesh,
                        super::pending_mesh_draw::PendingMeshGeometry::Skinned(primitive) => {
                            std::sync::Arc::new(GpuMeshResource::from_asset(device, primitive))
                        }
                    };
                    create_mesh_draw(
                        device,
                        model_layout,
                        mesh,
                        pending_draw.texture,
                        pending_draw.pipeline_key,
                        pending_draw.model_matrix,
                        pending_draw.draw_tint,
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
                },
            )
            .collect(),
        indirect_segment_count,
        indirect_args_count,
        indirect_args_buffer,
        indirect_submission_buffer,
        indirect_authority_buffer,
        indirect_draw_ref_buffer,
        indirect_segment_buffer,
    }
}

fn phase_ordered_meshes(
    frame: &ViewportRenderFrame,
) -> Vec<&crate::core::framework::render::RenderMeshSnapshot> {
    let phase_queue = &frame.extract.geometry.phase_queue;
    if phase_queue.items.is_empty() {
        return frame.meshes().iter().collect();
    }

    meshes_from_phase_queue(frame, phase_queue)
}

fn meshes_from_phase_queue<'a>(
    frame: &'a ViewportRenderFrame,
    phase_queue: &RenderPhaseQueue,
) -> Vec<&'a crate::core::framework::render::RenderMeshSnapshot> {
    phase_queue
        .items
        .iter()
        .filter_map(|item| match item.mesh_source {
            RenderPhaseMeshSource::MeshIndex(index) => frame.meshes().get(index),
        })
        .collect()
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
    use super::phase_ordered_meshes;
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
                GeometryPhaseInput {
                    entity: 30,
                    mesh_index: 0,
                    material_alpha_mode: RenderMaterialAlphaMode::Blend,
                    depth: 3.0,
                },
                GeometryPhaseInput {
                    entity: 10,
                    mesh_index: 1,
                    material_alpha_mode: RenderMaterialAlphaMode::Opaque,
                    depth: 1.0,
                },
                GeometryPhaseInput {
                    entity: 20,
                    mesh_index: 2,
                    material_alpha_mode: RenderMaterialAlphaMode::Mask { cutoff: 0.5 },
                    depth: 2.0,
                },
            ],
        );
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(320, 240));

        assert_eq!(
            phase_ordered_meshes(&frame)
                .into_iter()
                .map(|mesh| mesh.node_id)
                .collect::<Vec<_>>(),
            vec![10, 20, 30]
        );
    }

    fn test_mesh(node_id: u64) -> RenderMeshSnapshot {
        RenderMeshSnapshot {
            node_id,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(&format!(
                "builtin://test-model/{node_id}"
            ))),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                &format!("builtin://test-material/{node_id}"),
            )),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            render_layer_mask: u32::MAX,
        }
    }
}
