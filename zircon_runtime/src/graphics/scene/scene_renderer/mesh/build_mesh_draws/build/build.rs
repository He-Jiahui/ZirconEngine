use crate::graphics::scene::resources::GpuMeshResource;
use crate::graphics::types::ViewportRenderFrame;

use super::super::super::mesh_draw::MeshDraw;
use super::super::super::mesh_draw::VirtualGeometrySubmissionDetail;
use super::super::super::virtual_geometry_indirect_args_gpu_resources::VirtualGeometryIndirectArgsGpuResources;
use super::super::create_mesh_draw::create_mesh_draw;
use super::super::indexed_indirect_args::IndexedIndirectArgs;
use super::build_mesh_draw_build_context::build_mesh_draw_build_context;
use super::build_shared_indirect_args_buffer::build_shared_indirect_args_buffer;
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
    encoder: &mut wgpu::CommandEncoder,
    virtual_geometry_indirect_args: Option<&VirtualGeometryIndirectArgsGpuResources>,
    model_layout: &wgpu::BindGroupLayout,
    streamer: &crate::graphics::scene::resources::ResourceStreamer,
    frame: &ViewportRenderFrame,
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
    let authoritative_segment_keys = Vec::new();
    let authoritative_draw_refs = Vec::new();

    let shared_indirect_args_parts = virtual_geometry_indirect_args
        .filter(|_| virtual_geometry_enabled)
        .and_then(|virtual_geometry_indirect_args| {
            build_shared_indirect_args_buffer(
                device,
                encoder,
                virtual_geometry_indirect_args,
                &authoritative_segment_keys,
                &authoritative_draw_refs,
                &pending_draws,
            )
        })
        .map(|shared| shared.into_parts());
    let indirect_segment_count = shared_indirect_args_parts
        .as_ref()
        .map(|shared| shared.segment_count)
        .unwrap_or(0);
    let indirect_args_count = shared_indirect_args_parts
        .as_ref()
        .map(|shared| shared.args_count)
        .unwrap_or(0);
    let indirect_draw_ref_buffer = shared_indirect_args_parts
        .as_ref()
        .map(|shared| std::sync::Arc::clone(&shared.draw_ref_buffer));
    let indirect_submission_buffer = shared_indirect_args_parts
        .as_ref()
        .map(|shared| std::sync::Arc::clone(&shared.submission_buffer));
    let indirect_authority_buffer = shared_indirect_args_parts
        .as_ref()
        .map(|shared| std::sync::Arc::clone(&shared.authority_buffer));
    let indirect_segment_buffer = shared_indirect_args_parts
        .as_ref()
        .map(|shared| std::sync::Arc::clone(&shared.segment_buffer));
    let indirect_args_offsets = shared_indirect_args_parts
        .as_ref()
        .map(|shared| shared.indirect_args_offsets.clone())
        .unwrap_or_default();
    let pending_draw_draw_ref_indices = shared_indirect_args_parts
        .as_ref()
        .map(|shared| shared.pending_draw_draw_ref_indices.clone())
        .unwrap_or_default();
    let pending_draw_submission_tokens = shared_indirect_args_parts
        .as_ref()
        .map(|shared| shared.pending_draw_submission_tokens.clone())
        .unwrap_or_default();
    let pending_draw_submission_details = shared_indirect_args_parts
        .as_ref()
        .map(|shared| shared.pending_draw_submission_details.clone())
        .unwrap_or_default();
    let pending_draw_submission_plan = shared_indirect_args_parts
        .as_ref()
        .map(|shared| shared.pending_draw_submission_plan.clone())
        .unwrap_or_default();
    let shared_indirect_args_buffer = shared_indirect_args_parts
        .as_ref()
        .map(|shared| std::sync::Arc::clone(&shared.buffer));
    let indirect_args_buffer = shared_indirect_args_buffer.clone();
    let indirect_args_stride = std::mem::size_of::<IndexedIndirectArgs>() as u64;

    let mut pending_draws = pending_draws.into_iter().map(Some).collect::<Vec<_>>();
    let mut ordered_pending_draws = Vec::new();
    if shared_indirect_args_buffer.is_some() {
        for submission in &pending_draw_submission_plan {
            let Some(pending_draw) = pending_draws
                .get_mut(submission.pending_draw_index())
                .and_then(Option::take)
            else {
                continue;
            };
            ordered_pending_draws.push((
                submission.indirect_args_offset(),
                submission.pending_draw_index(),
                Some(submission.submission_detail()),
                pending_draw,
            ));
        }
    }
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
