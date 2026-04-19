use crate::graphics::types::EditorOrRuntimeFrame;

use super::super::super::mesh_draw::MeshDraw;
use super::super::super::mesh_draw::VirtualGeometrySubmissionDetail;
use super::super::super::virtual_geometry_indirect_args_gpu_resources::VirtualGeometryIndirectArgsGpuResources;
use super::super::create_mesh_draw::create_mesh_draw;
use super::super::indexed_indirect_args::IndexedIndirectArgs;
use super::build_mesh_draw_build_context::build_mesh_draw_build_context;
use super::build_shared_indirect_args_buffer::build_shared_indirect_args_buffer;
use super::extend_pending_draws_for_mesh_instance::extend_pending_draws_for_mesh_instance;
use super::pending_mesh_draw::{indirect_draw_ref_for_cluster_draw, segment_key_for_cluster_draw};

pub(crate) struct BuiltMeshDraws {
    pub(crate) draws: Vec<MeshDraw>,
    pub(crate) indirect_segment_count: u32,
    pub(crate) indirect_args_count: u32,
    pub(crate) indirect_submission_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    pub(crate) indirect_draw_ref_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    pub(crate) indirect_segment_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
}

pub(crate) fn build_mesh_draws(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    virtual_geometry_indirect_args: &VirtualGeometryIndirectArgsGpuResources,
    model_layout: &wgpu::BindGroupLayout,
    streamer: &crate::graphics::scene::resources::ResourceStreamer,
    frame: &EditorOrRuntimeFrame,
    virtual_geometry_enabled: bool,
) -> BuiltMeshDraws {
    let build_context = build_mesh_draw_build_context(frame, virtual_geometry_enabled);
    let mut pending_draws = Vec::new();
    for mesh_instance in &frame.scene.scene.meshes {
        extend_pending_draws_for_mesh_instance(
            &mut pending_draws,
            streamer,
            frame,
            &build_context,
            mesh_instance,
        );
    }
    let authoritative_segment_keys = build_context
        .virtual_geometry_cluster_draws
        .as_ref()
        .map(|cluster_draws| {
            cluster_draws
                .iter()
                .flat_map(|(entity, cluster_draws)| {
                    cluster_draws
                        .iter()
                        .copied()
                        .map(|cluster_draw| segment_key_for_cluster_draw(*entity, cluster_draw))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let authoritative_draw_refs = build_context
        .virtual_geometry_cluster_draws
        .as_ref()
        .map(|cluster_draws| {
            frame
                .scene
                .scene
                .meshes
                .iter()
                .flat_map(|mesh_instance| {
                    let Some(entity_cluster_draws) = cluster_draws.get(&mesh_instance.node_id)
                    else {
                        return Vec::new();
                    };
                    let Some(model) = streamer.model(&mesh_instance.model.id()) else {
                        return Vec::new();
                    };
                    model
                        .meshes
                        .iter()
                        .flat_map(|mesh| {
                            entity_cluster_draws
                                .iter()
                                .copied()
                                .map(move |cluster_draw| {
                                    indirect_draw_ref_for_cluster_draw(
                                        mesh_instance.node_id,
                                        mesh.index_count,
                                        mesh.indirect_order_signature,
                                        cluster_draw,
                                    )
                                })
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let shared_indirect_args_buffer = virtual_geometry_enabled
        .then(|| {
            build_shared_indirect_args_buffer(
                device,
                encoder,
                virtual_geometry_indirect_args,
                &authoritative_segment_keys,
                &authoritative_draw_refs,
                &pending_draws,
            )
        })
        .flatten();
    let indirect_segment_count = shared_indirect_args_buffer
        .as_ref()
        .map(|shared| shared.segment_count)
        .unwrap_or(0);
    let indirect_args_count = shared_indirect_args_buffer
        .as_ref()
        .map(|shared| shared.args_count)
        .unwrap_or(0);
    let indirect_draw_ref_buffer = shared_indirect_args_buffer
        .as_ref()
        .map(|shared| std::sync::Arc::clone(&shared.draw_ref_buffer));
    let indirect_submission_buffer = shared_indirect_args_buffer
        .as_ref()
        .map(|shared| std::sync::Arc::clone(&shared.submission_buffer));
    let indirect_segment_buffer = shared_indirect_args_buffer
        .as_ref()
        .map(|shared| std::sync::Arc::clone(&shared.segment_buffer));
    let indirect_args_offsets = shared_indirect_args_buffer
        .as_ref()
        .map(|shared| shared.indirect_args_offsets.clone())
        .unwrap_or_default();
    let pending_draw_submission_orders = shared_indirect_args_buffer
        .as_ref()
        .map(|shared| shared.pending_draw_submission_orders.clone())
        .unwrap_or_default();
    let pending_draw_submission_tokens = shared_indirect_args_buffer
        .as_ref()
        .map(|shared| shared.pending_draw_submission_tokens.clone())
        .unwrap_or_default();
    let pending_draw_submission_details = shared_indirect_args_buffer
        .as_ref()
        .map(|shared| shared.pending_draw_submission_details.clone())
        .unwrap_or_default();
    let shared_indirect_args_buffer = shared_indirect_args_buffer.map(|shared| shared.buffer);
    let indirect_args_stride = std::mem::size_of::<IndexedIndirectArgs>() as u64;

    let mut ordered_pending_draws = pending_draws
        .into_iter()
        .enumerate()
        .map(|(index, pending_draw)| {
            (
                pending_draw_submission_orders
                    .get(index)
                    .copied()
                    .unwrap_or(index as u32),
                pending_draw_submission_tokens
                    .get(index)
                    .copied()
                    .unwrap_or(u32::MAX),
                indirect_args_offsets
                    .get(index)
                    .copied()
                    .unwrap_or((index as u64) * indirect_args_stride),
                index,
                pending_draw,
            )
        })
        .collect::<Vec<_>>();
    if shared_indirect_args_buffer.is_some() {
        ordered_pending_draws.sort_by_key(
            |(
                submission_order,
                submission_token,
                indirect_args_offset,
                original_index,
                _pending_draw,
            )| {
                (
                    *submission_order,
                    *submission_token,
                    *indirect_args_offset,
                    *original_index,
                )
            },
        );
    }
    BuiltMeshDraws {
        draws: ordered_pending_draws
            .into_iter()
            .map(
                |(
                    _submission_order,
                    submission_token,
                    indirect_args_offset,
                    _original_index,
                    pending_draw,
                )| {
                    create_mesh_draw(
                        device,
                        model_layout,
                        pending_draw.mesh,
                        pending_draw.texture,
                        pending_draw.pipeline_key,
                        pending_draw.model_matrix,
                        pending_draw.draw_tint,
                        pending_draw.first_index,
                        pending_draw.draw_index_count,
                        shared_indirect_args_buffer.clone(),
                        indirect_args_offset,
                        pending_draw_submission_details
                            .get(_original_index)
                            .copied()
                            .flatten()
                            .or_else(|| {
                                pending_draw.indirect_draw_ref.map(|draw_ref| {
                                    VirtualGeometrySubmissionDetail {
                                        entity: draw_ref.segment_key.entity,
                                        page_id: draw_ref.segment_key.page_id,
                                        submission_index: if submission_token == u32::MAX {
                                            0
                                        } else {
                                            submission_token >> 16
                                        },
                                        draw_ref_rank: if submission_token == u32::MAX {
                                            0
                                        } else {
                                            submission_token & 0xffff
                                        },
                                    }
                                })
                            }),
                    )
                },
            )
            .collect(),
        indirect_segment_count,
        indirect_args_count,
        indirect_submission_buffer,
        indirect_draw_ref_buffer,
        indirect_segment_buffer,
    }
}
