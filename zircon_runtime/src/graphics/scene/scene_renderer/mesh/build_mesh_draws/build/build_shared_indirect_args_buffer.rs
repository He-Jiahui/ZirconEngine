use std::collections::HashMap;
use std::sync::Arc;

use wgpu::util::DeviceExt;

use super::super::super::mesh_draw::VirtualGeometrySubmissionDetail;
use super::super::super::virtual_geometry_indirect_args_gpu_resources::VirtualGeometryIndirectArgsGpuResources;
use super::super::indexed_indirect_args::IndexedIndirectArgs;
use super::pending_mesh_draw::{
    draw_ref_input, segment_input, PendingMeshDraw, VirtualGeometryIndirectDrawRef,
    VirtualGeometryIndirectDrawRefInput, VirtualGeometryIndirectSegmentInput,
    VirtualGeometryIndirectSegmentKey,
};

pub(super) struct SharedIndirectArgsBuffer {
    pub(super) buffer: Arc<wgpu::Buffer>,
    pub(super) submission_buffer: Arc<wgpu::Buffer>,
    pub(super) draw_ref_buffer: Arc<wgpu::Buffer>,
    pub(super) segment_buffer: Arc<wgpu::Buffer>,
    pub(super) segment_count: u32,
    pub(super) args_count: u32,
    pub(super) indirect_args_offsets: Vec<u64>,
    pub(super) pending_draw_submission_orders: Vec<u32>,
    pub(super) pending_draw_submission_tokens: Vec<u32>,
    pub(super) pending_draw_submission_details: Vec<Option<VirtualGeometrySubmissionDetail>>,
}

pub(super) fn build_shared_indirect_args_buffer(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    virtual_geometry_indirect_args: &VirtualGeometryIndirectArgsGpuResources,
    authoritative_segment_keys: &[VirtualGeometryIndirectSegmentKey],
    authoritative_draw_refs: &[VirtualGeometryIndirectDrawRef],
    pending_draws: &[PendingMeshDraw],
) -> Option<SharedIndirectArgsBuffer> {
    let pending_draw_refs = pending_draws
        .iter()
        .enumerate()
        .filter_map(|(pending_draw_index, pending_draw)| {
            pending_draw
                .indirect_draw_ref
                .map(|draw_ref| (pending_draw_index, draw_ref))
        })
        .collect::<Vec<_>>();
    let layout = build_shared_indirect_args_layout(
        authoritative_segment_keys,
        authoritative_draw_refs,
        &pending_draw_refs,
        pending_draws.len(),
    )?;
    let segment_count = layout.segment_inputs.len() as u32;
    let args_count = layout.draw_refs.len() as u32;

    let segment_buffer = Arc::new(segment_buffer(device, &layout.segment_inputs));
    let draw_ref_buffer = Arc::new(draw_ref_buffer(device, &layout.draw_refs));
    let output_buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-vg-indirect-args"),
        size: (layout.draw_refs.len() * std::mem::size_of::<IndexedIndirectArgs>()) as u64,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::INDIRECT
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    }));
    let submission_buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-vg-indirect-submission-tokens"),
        size: (layout.draw_refs.len() * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    }));
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-vg-indirect-args-bind-group"),
        layout: &virtual_geometry_indirect_args.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: segment_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: draw_ref_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: output_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: submission_buffer.as_entire_binding(),
            },
        ],
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("zircon-vg-indirect-args-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&virtual_geometry_indirect_args.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(args_count.div_ceil(64), 1, 1);
    }

    Some(SharedIndirectArgsBuffer {
        buffer: output_buffer,
        submission_buffer,
        draw_ref_buffer,
        segment_buffer,
        segment_count,
        args_count,
        indirect_args_offsets: layout.indirect_args_offsets,
        pending_draw_submission_orders: layout.pending_draw_submission_orders,
        pending_draw_submission_tokens: layout.pending_draw_submission_tokens,
        pending_draw_submission_details: layout.pending_draw_submission_details,
    })
}

struct SharedIndirectArgsLayout {
    segment_inputs: Vec<VirtualGeometryIndirectSegmentInput>,
    draw_refs: Vec<VirtualGeometryIndirectDrawRefInput>,
    indirect_args_offsets: Vec<u64>,
    pending_draw_submission_orders: Vec<u32>,
    pending_draw_submission_tokens: Vec<u32>,
    pending_draw_submission_details: Vec<Option<VirtualGeometrySubmissionDetail>>,
}

fn build_shared_indirect_args_layout(
    authoritative_segment_keys: &[VirtualGeometryIndirectSegmentKey],
    authoritative_draw_refs: &[VirtualGeometryIndirectDrawRef],
    pending_draw_refs: &[(usize, VirtualGeometryIndirectDrawRef)],
    pending_draw_count: usize,
) -> Option<SharedIndirectArgsLayout> {
    let mut ordered_draw_refs = pending_draw_refs
        .iter()
        .copied()
        .map(|(pending_draw_index, draw_ref)| OrderedDrawRef {
            pending_draw_index,
            draw_ref,
        })
        .collect::<Vec<_>>();

    if ordered_draw_refs.is_empty() && authoritative_draw_refs.is_empty() {
        return None;
    }

    ordered_draw_refs.sort_by_key(draw_ref_submission_order_key);
    let mut authoritative_ordered_draw_refs = authoritative_draw_refs
        .iter()
        .copied()
        .enumerate()
        .map(|(authoritative_index, draw_ref)| AuthoritativeDrawRef {
            authoritative_index,
            draw_ref,
        })
        .collect::<Vec<_>>();
    authoritative_ordered_draw_refs.sort_by_key(authoritative_draw_ref_submission_order_key);

    let mut unique_segment_keys = authoritative_segment_keys.to_vec();
    unique_segment_keys.extend(
        ordered_draw_refs
            .iter()
            .map(|ordered_draw_ref| ordered_draw_ref.draw_ref.segment_key),
    );
    unique_segment_keys.sort_by_key(segment_submission_order_key);
    unique_segment_keys.dedup();

    let segment_indices = unique_segment_keys
        .iter()
        .copied()
        .enumerate()
        .map(|(segment_index, segment_key)| (segment_key, segment_index as u32))
        .collect::<HashMap<_, _>>();
    let segment_inputs = unique_segment_keys
        .iter()
        .copied()
        .map(segment_input)
        .collect::<Vec<VirtualGeometryIndirectSegmentInput>>();
    let indirect_args_stride = std::mem::size_of::<IndexedIndirectArgs>() as u64;
    let mut indirect_args_offsets = vec![0_u64; pending_draw_count];
    let mut pending_draw_submission_orders = vec![u32::MAX; pending_draw_count];
    let mut pending_draw_draw_ref_indices = vec![u32::MAX; pending_draw_count];
    let mut indirect_args_keys = HashMap::<IndirectArgsKey, u32>::new();
    let mut authoritative_occurrence_ranks = HashMap::<DrawRefGroupKey, u32>::new();
    let mut draw_ref_entries = Vec::<DrawRefEntry>::new();
    for authoritative_draw_ref in authoritative_ordered_draw_refs {
        let group_key = DrawRefGroupKey {
            mesh_index_count: authoritative_draw_ref.draw_ref.mesh_index_count,
            mesh_signature: authoritative_draw_ref.draw_ref.mesh_signature,
            segment_key: authoritative_draw_ref.draw_ref.segment_key,
        };
        let occurrence_rank = next_occurrence_rank(&mut authoritative_occurrence_ranks, group_key);
        let indirect_args_key = IndirectArgsKey {
            group_key,
            occurrence_rank,
        };
        if indirect_args_keys.contains_key(&indirect_args_key) {
            continue;
        }
        let segment_index = segment_indices[&authoritative_draw_ref.draw_ref.segment_key];
        let next_index = draw_ref_entries.len() as u32;
        draw_ref_entries.push(DrawRefEntry {
            mesh_index_count: authoritative_draw_ref.draw_ref.mesh_index_count,
            segment_index,
        });
        indirect_args_keys.insert(indirect_args_key, next_index);
    }
    let mut pending_occurrence_ranks = HashMap::<DrawRefGroupKey, u32>::new();
    for (submission_order, ordered_draw_ref) in ordered_draw_refs.into_iter().enumerate() {
        let group_key = DrawRefGroupKey {
            mesh_index_count: ordered_draw_ref.draw_ref.mesh_index_count,
            mesh_signature: ordered_draw_ref.draw_ref.mesh_signature,
            segment_key: ordered_draw_ref.draw_ref.segment_key,
        };
        let occurrence_rank = next_occurrence_rank(&mut pending_occurrence_ranks, group_key);
        let indirect_args_key = IndirectArgsKey {
            group_key,
            occurrence_rank,
        };
        let draw_ref_index = match indirect_args_keys.get(&indirect_args_key).copied() {
            Some(existing_index) => existing_index,
            None => {
                let segment_index = segment_indices[&ordered_draw_ref.draw_ref.segment_key];
                let next_index = draw_ref_entries.len() as u32;
                draw_ref_entries.push(DrawRefEntry {
                    mesh_index_count: ordered_draw_ref.draw_ref.mesh_index_count,
                    segment_index,
                });
                indirect_args_keys.insert(indirect_args_key, next_index);
                next_index
            }
        };
        indirect_args_offsets[ordered_draw_ref.pending_draw_index] =
            (draw_ref_index as u64) * indirect_args_stride;
        pending_draw_submission_orders[ordered_draw_ref.pending_draw_index] =
            submission_order as u32;
        pending_draw_draw_ref_indices[ordered_draw_ref.pending_draw_index] = draw_ref_index;
    }

    if draw_ref_entries.is_empty() {
        return None;
    }

    let draw_ref_ranks = draw_ref_ranks_within_segment(&draw_ref_entries);
    let draw_ref_counts = draw_ref_counts_within_segment(&draw_ref_entries);
    let draw_ref_submission_tokens = draw_ref_entries
        .iter()
        .enumerate()
        .map(|(draw_ref_index, draw_ref)| {
            let segment_key = unique_segment_keys[draw_ref.segment_index as usize];
            (segment_key.submission_index.min(0xffff) << 16)
                | draw_ref_ranks[draw_ref_index].min(0xffff)
        })
        .collect::<Vec<_>>();
    let draw_refs = draw_ref_entries
        .iter()
        .enumerate()
        .map(|(draw_ref_index, draw_ref)| {
            draw_ref_input(
                draw_ref.mesh_index_count,
                draw_ref.segment_index,
                draw_ref_counts[draw_ref_index],
                draw_ref_submission_tokens[draw_ref_index],
            )
        })
        .collect::<Vec<_>>();
    let pending_draw_submission_tokens = pending_draw_draw_ref_indices
        .iter()
        .copied()
        .map(|draw_ref_index| {
            if draw_ref_index == u32::MAX {
                return u32::MAX;
            }
            draw_ref_submission_tokens[draw_ref_index as usize]
        })
        .collect::<Vec<_>>();
    let pending_draw_submission_details = pending_draw_draw_ref_indices
        .iter()
        .copied()
        .map(|draw_ref_index| {
            if draw_ref_index == u32::MAX {
                return None;
            }
            let draw_ref = draw_ref_entries[draw_ref_index as usize];
            let segment_key = unique_segment_keys[draw_ref.segment_index as usize];
            let submission_token = draw_ref_submission_tokens[draw_ref_index as usize];
            Some(VirtualGeometrySubmissionDetail {
                entity: segment_key.entity,
                page_id: segment_key.page_id,
                submission_index: submission_token >> 16,
                draw_ref_rank: submission_token & 0xffff,
            })
        })
        .collect::<Vec<_>>();

    Some(SharedIndirectArgsLayout {
        segment_inputs,
        draw_refs,
        indirect_args_offsets,
        pending_draw_submission_orders,
        pending_draw_submission_tokens,
        pending_draw_submission_details,
    })
}

fn draw_ref_ranks_within_segment(draw_refs: &[DrawRefEntry]) -> Vec<u32> {
    let mut next_rank_by_segment = HashMap::<u32, u32>::new();
    draw_refs
        .iter()
        .map(|draw_ref| {
            let next_rank = next_rank_by_segment
                .entry(draw_ref.segment_index)
                .or_insert(0);
            let rank = *next_rank;
            *next_rank += 1;
            rank
        })
        .collect()
}

fn draw_ref_counts_within_segment(draw_refs: &[DrawRefEntry]) -> Vec<u32> {
    let mut count_by_segment = HashMap::<u32, u32>::new();
    for draw_ref in draw_refs {
        *count_by_segment.entry(draw_ref.segment_index).or_insert(0) += 1;
    }

    draw_refs
        .iter()
        .map(|draw_ref| {
            count_by_segment
                .get(&draw_ref.segment_index)
                .copied()
                .unwrap_or(1)
        })
        .collect()
}

#[derive(Clone, Copy)]
struct DrawRefEntry {
    mesh_index_count: u32,
    segment_index: u32,
}

#[derive(Clone, Copy)]
struct OrderedDrawRef {
    pending_draw_index: usize,
    draw_ref: VirtualGeometryIndirectDrawRef,
}

#[derive(Clone, Copy)]
struct AuthoritativeDrawRef {
    authoritative_index: usize,
    draw_ref: VirtualGeometryIndirectDrawRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct IndirectArgsKey {
    group_key: DrawRefGroupKey,
    occurrence_rank: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct DrawRefGroupKey {
    mesh_index_count: u32,
    mesh_signature: u64,
    segment_key: VirtualGeometryIndirectSegmentKey,
}

fn next_occurrence_rank(
    occurrence_ranks: &mut HashMap<DrawRefGroupKey, u32>,
    group_key: DrawRefGroupKey,
) -> u32 {
    let next_rank = occurrence_ranks.entry(group_key).or_insert(0);
    let rank = *next_rank;
    *next_rank += 1;
    rank
}

fn draw_ref_submission_order_key(
    ordered_draw_ref: &OrderedDrawRef,
) -> (
    (u32, u32, u32, u64, u32, u32),
    (u32, u32, u8, u32, u32, u32, u64),
    usize,
) {
    let segment_key = ordered_draw_ref.draw_ref.segment_key;
    (
        (
            segment_key.submission_index,
            segment_key.submission_slot.unwrap_or(u32::MAX),
            segment_key.frontier_rank,
            segment_key.entity,
            segment_key.cluster_start_ordinal,
            segment_key.page_id,
        ),
        (
            segment_key.cluster_span_count,
            segment_key.cluster_total_count,
            segment_key.lod_level,
            segment_key.lineage_depth,
            segment_key.state,
            ordered_draw_ref.draw_ref.mesh_index_count,
            ordered_draw_ref.draw_ref.mesh_signature,
        ),
        ordered_draw_ref.pending_draw_index,
    )
}

fn authoritative_draw_ref_submission_order_key(
    draw_ref: &AuthoritativeDrawRef,
) -> (
    (u32, u32, u32, u64, u32, u32),
    (u32, u32, u8, u32, u32, u32, u64),
    usize,
) {
    let segment_key = draw_ref.draw_ref.segment_key;
    (
        (
            segment_key.submission_index,
            segment_key.submission_slot.unwrap_or(u32::MAX),
            segment_key.frontier_rank,
            segment_key.entity,
            segment_key.cluster_start_ordinal,
            segment_key.page_id,
        ),
        (
            segment_key.cluster_span_count,
            segment_key.cluster_total_count,
            segment_key.lod_level,
            segment_key.lineage_depth,
            segment_key.state,
            draw_ref.draw_ref.mesh_index_count,
            draw_ref.draw_ref.mesh_signature,
        ),
        draw_ref.authoritative_index,
    )
}

fn segment_submission_order_key(
    segment_key: &VirtualGeometryIndirectSegmentKey,
) -> ((u32, u32, u32, u64, u32, u32, u32, u32, u8, u32), u32) {
    (
        (
            segment_key.submission_index,
            segment_key.submission_slot.unwrap_or(u32::MAX),
            segment_key.frontier_rank,
            segment_key.entity,
            segment_key.cluster_start_ordinal,
            segment_key.page_id,
            segment_key.cluster_span_count,
            segment_key.cluster_total_count,
            segment_key.lod_level,
            segment_key.lineage_depth,
        ),
        segment_key.state,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_shared_indirect_args_layout, VirtualGeometryIndirectDrawRef,
        VirtualGeometryIndirectSegmentKey,
    };
    use crate::graphics::scene::scene_renderer::mesh::mesh_draw::VirtualGeometrySubmissionDetail;

    #[test]
    fn shared_indirect_args_layout_keeps_distinct_gpu_args_slots_for_repeated_draw_refs() {
        let segment_a = segment_key(0, 2, 300);
        let segment_b = segment_key(1, 3, 301);
        let draw_ref_a = VirtualGeometryIndirectDrawRef {
            mesh_index_count: 6,
            mesh_signature: 11,
            segment_key: segment_a,
        };
        let layout = build_shared_indirect_args_layout(
            &[segment_a, segment_b],
            &[],
            &[
                (0, draw_ref_a),
                (1, draw_ref_a),
                (
                    2,
                    VirtualGeometryIndirectDrawRef {
                        mesh_index_count: 6,
                        mesh_signature: 17,
                        segment_key: segment_b,
                    },
                ),
            ],
            3,
        )
        .expect("expected shared indirect args layout");

        assert_eq!(
            layout.pending_draw_submission_orders,
            vec![0, 1, 2],
            "expected layout to keep one visibility-owned submission rank per pending draw even when repeated draw refs now keep distinct GPU args authority"
        );
        assert_eq!(
            layout.pending_draw_submission_tokens,
            vec![0, 1, 1 << 16],
            "expected repeated draw refs on the same visibility-owned segment to keep unique per-draw submission tokens instead of collapsing back onto one shared args slot"
        );
        assert_eq!(
            layout
                .draw_refs
                .iter()
                .map(|draw_ref| (
                    draw_ref.mesh_index_count,
                    draw_ref.segment_index,
                    draw_ref.segment_draw_ref_count,
                    draw_ref.submission_token,
                ))
                .collect::<Vec<_>>(),
            vec![(6, 0, 2, 0), (6, 0, 2, 1), (6, 1, 1, 1 << 16)],
            "expected the GPU draw-ref buffer itself to carry explicit draw-ref count and submission-token truth so cluster-raster args no longer need to reconstruct both values from buffer iteration order"
        );
        assert_eq!(
            layout.pending_draw_submission_details,
            vec![
                Some(VirtualGeometrySubmissionDetail {
                    entity: 2,
                    page_id: 300,
                    submission_index: 0,
                    draw_ref_rank: 0,
                }),
                Some(VirtualGeometrySubmissionDetail {
                    entity: 2,
                    page_id: 300,
                    submission_index: 0,
                    draw_ref_rank: 1,
                }),
                Some(VirtualGeometrySubmissionDetail {
                    entity: 3,
                    page_id: 301,
                    submission_index: 1,
                    draw_ref_rank: 0,
                }),
            ],
            "expected renderer-side submission detail to be derived from the same shared layout truth that authored the GPU draw-ref input instead of being reconstructed later from per-draw CPU residue"
        );
        assert!(
            layout.indirect_args_offsets[1] > layout.indirect_args_offsets[0],
            "expected repeated draw refs to keep distinct shared indirect args offsets so GPU-generated args source, not only CPU ordering, remains authoritative"
        );
        assert!(
            layout.indirect_args_offsets[2] > layout.indirect_args_offsets[1],
            "expected the later distinct segment to keep a later shared indirect args record after repeated draw refs already claimed independent GPU args slots"
        );
    }

    #[test]
    fn shared_indirect_args_layout_keeps_mesh_signature_in_authoritative_compaction_key() {
        let segment = segment_key(0, 2, 300);
        let authoritative_signature_a = VirtualGeometryIndirectDrawRef {
            mesh_index_count: 6,
            mesh_signature: 11,
            segment_key: segment,
        };
        let authoritative_signature_b = VirtualGeometryIndirectDrawRef {
            mesh_index_count: 6,
            mesh_signature: 17,
            segment_key: segment,
        };
        let layout = build_shared_indirect_args_layout(
            &[segment],
            &[authoritative_signature_a, authoritative_signature_b],
            &[(0, authoritative_signature_b)],
            1,
        )
        .expect("expected shared indirect args layout");

        assert_eq!(
            layout.pending_draw_submission_tokens,
            vec![1],
            "expected authoritative indirect compaction to keep mesh-signature identity in the shared args key so a surviving later primitive does not get remapped onto an earlier primitive's args slot just because they share segment/page/index-count truth"
        );
        assert_eq!(
            layout.pending_draw_submission_details,
            vec![Some(VirtualGeometrySubmissionDetail {
                entity: 2,
                page_id: 300,
                submission_index: 0,
                draw_ref_rank: 1,
            })],
            "expected renderer-side submission detail to keep the later authoritative draw-ref rank when only that primitive survives the drawable subset, instead of collapsing back onto the earlier primitive's CPU-compacted slot"
        );
    }

    fn segment_key(
        submission_index: u32,
        entity: u64,
        page_id: u32,
    ) -> VirtualGeometryIndirectSegmentKey {
        VirtualGeometryIndirectSegmentKey {
            submission_index,
            entity,
            page_id,
            cluster_start_ordinal: 0,
            cluster_span_count: 1,
            cluster_total_count: 1,
            lineage_depth: 0,
            lod_level: 0,
            frontier_rank: 0,
            submission_slot: Some(submission_index + 1),
            state: 0,
        }
    }
}

fn segment_buffer(
    device: &wgpu::Device,
    segment_inputs: &[VirtualGeometryIndirectSegmentInput],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-vg-indirect-segments"),
        contents: bytemuck::cast_slice(segment_inputs),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    })
}

fn draw_ref_buffer(
    device: &wgpu::Device,
    draw_refs: &[VirtualGeometryIndirectDrawRefInput],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-vg-indirect-draw-refs"),
        contents: bytemuck::cast_slice(draw_refs),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    })
}
