use std::collections::HashMap;
use std::sync::Arc;

use wgpu::util::DeviceExt;

use super::super::super::virtual_geometry_indirect_args_gpu_resources::VirtualGeometryIndirectArgsGpuResources;
use super::super::indexed_indirect_args::IndexedIndirectArgs;
use super::pending_mesh_draw::{
    draw_ref_input, segment_input, PendingMeshDraw, VirtualGeometryIndirectDrawRef,
    VirtualGeometryIndirectDrawRefInput,
    VirtualGeometryIndirectSegmentInput, VirtualGeometryIndirectSegmentKey,
};

pub(super) struct SharedIndirectArgsBuffer {
    pub(super) buffer: Arc<wgpu::Buffer>,
    pub(super) draw_ref_buffer: Arc<wgpu::Buffer>,
    pub(super) segment_buffer: Arc<wgpu::Buffer>,
    pub(super) segment_count: u32,
    pub(super) args_count: u32,
    pub(super) indirect_args_offsets: Vec<u64>,
}

pub(super) fn build_shared_indirect_args_buffer(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    virtual_geometry_indirect_args: &VirtualGeometryIndirectArgsGpuResources,
    pending_draws: &[PendingMeshDraw],
) -> Option<SharedIndirectArgsBuffer> {
    let mut ordered_draw_refs = pending_draws
        .iter()
        .enumerate()
        .filter_map(|(pending_draw_index, pending_draw)| {
            pending_draw.indirect_draw_ref.map(|draw_ref| OrderedDrawRef {
                pending_draw_index,
                draw_ref,
            })
        })
        .collect::<Vec<_>>();

    if ordered_draw_refs.is_empty() {
        return None;
    }

    ordered_draw_refs.sort_by_key(draw_ref_submission_order_key);

    let mut unique_segment_keys = ordered_draw_refs
        .iter()
        .map(|ordered_draw_ref| ordered_draw_ref.draw_ref.segment_key)
        .collect::<Vec<_>>();
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
    let mut indirect_args_offsets = vec![0_u64; pending_draws.len()];
    let mut indirect_args_keys = HashMap::<IndirectArgsKey, u32>::new();
    let mut draw_refs = Vec::<VirtualGeometryIndirectDrawRefInput>::new();
    for ordered_draw_ref in ordered_draw_refs {
        let indirect_args_key = IndirectArgsKey {
            mesh_index_count: ordered_draw_ref.draw_ref.mesh_index_count,
            segment_key: ordered_draw_ref.draw_ref.segment_key,
        };
        let draw_ref_index = match indirect_args_keys.get(&indirect_args_key).copied() {
            Some(existing_index) => existing_index,
            None => {
                let segment_index = segment_indices[&ordered_draw_ref.draw_ref.segment_key];
                let next_index = draw_refs.len() as u32;
                draw_refs.push(draw_ref_input(ordered_draw_ref.draw_ref, segment_index));
                indirect_args_keys.insert(indirect_args_key, next_index);
                next_index
            }
        };
        indirect_args_offsets[ordered_draw_ref.pending_draw_index] =
            (draw_ref_index as u64) * indirect_args_stride;
    }

    if draw_refs.is_empty() {
        return None;
    }

    let segment_buffer = Arc::new(segment_buffer(device, &segment_inputs));
    let draw_ref_buffer = Arc::new(draw_ref_buffer(device, &draw_refs));
    let output_buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-vg-indirect-args"),
        size: (draw_refs.len() * std::mem::size_of::<IndexedIndirectArgs>()) as u64,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::INDIRECT
            | wgpu::BufferUsages::COPY_SRC,
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
        ],
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("zircon-vg-indirect-args-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&virtual_geometry_indirect_args.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups((draw_refs.len() as u32).div_ceil(64), 1, 1);
    }

    Some(SharedIndirectArgsBuffer {
        buffer: output_buffer,
        draw_ref_buffer,
        segment_buffer,
        segment_count: segment_inputs.len() as u32,
        args_count: draw_refs.len() as u32,
        indirect_args_offsets,
    })
}

#[derive(Clone, Copy)]
struct OrderedDrawRef {
    pending_draw_index: usize,
    draw_ref: VirtualGeometryIndirectDrawRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct IndirectArgsKey {
    mesh_index_count: u32,
    segment_key: VirtualGeometryIndirectSegmentKey,
}

fn draw_ref_submission_order_key(
    ordered_draw_ref: &OrderedDrawRef,
) -> (
    (u32, u32, u32, u64, u32, u32),
    (u32, u32, u8, u32, u32, u32),
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
        ),
        ordered_draw_ref.pending_draw_index,
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
