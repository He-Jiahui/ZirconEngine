use std::collections::HashMap;
use std::sync::Arc;

use wgpu::util::DeviceExt;

use super::super::super::virtual_geometry_indirect_args_gpu_resources::VirtualGeometryIndirectArgsGpuResources;
use super::super::indexed_indirect_args::IndexedIndirectArgs;
use super::pending_mesh_draw::{
    draw_ref_input, segment_input, PendingMeshDraw, VirtualGeometryIndirectDrawRefInput,
    VirtualGeometryIndirectSegmentInput, VirtualGeometryIndirectSegmentKey,
};

pub(super) struct SharedIndirectArgsBuffer {
    pub(super) buffer: Arc<wgpu::Buffer>,
    pub(super) segment_count: u32,
}

pub(super) fn build_shared_indirect_args_buffer(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    virtual_geometry_indirect_args: &VirtualGeometryIndirectArgsGpuResources,
    pending_draws: &[PendingMeshDraw],
) -> Option<SharedIndirectArgsBuffer> {
    let mut segment_indices = HashMap::<VirtualGeometryIndirectSegmentKey, u32>::new();
    let mut segment_inputs = Vec::<VirtualGeometryIndirectSegmentInput>::new();
    let mut draw_refs = Vec::<VirtualGeometryIndirectDrawRefInput>::new();

    for pending_draw in pending_draws {
        let Some(indirect_draw_ref) = pending_draw.indirect_draw_ref else {
            continue;
        };
        let segment_index =
            if let Some(&segment_index) = segment_indices.get(&indirect_draw_ref.segment_key) {
                segment_index
            } else {
                let segment_index = segment_inputs.len() as u32;
                segment_indices.insert(indirect_draw_ref.segment_key, segment_index);
                segment_inputs.push(segment_input(indirect_draw_ref.segment_key));
                segment_index
            };
        draw_refs.push(draw_ref_input(indirect_draw_ref, segment_index));
    }

    if draw_refs.is_empty() {
        return None;
    }

    let segment_buffer = segment_buffer(device, &segment_inputs);
    let draw_ref_buffer = draw_ref_buffer(device, &draw_refs);
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
        segment_count: segment_inputs.len() as u32,
    })
}

fn segment_buffer(
    device: &wgpu::Device,
    segment_inputs: &[VirtualGeometryIndirectSegmentInput],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-vg-indirect-segments"),
        contents: bytemuck::cast_slice(segment_inputs),
        usage: wgpu::BufferUsages::STORAGE,
    })
}

fn draw_ref_buffer(
    device: &wgpu::Device,
    draw_refs: &[VirtualGeometryIndirectDrawRefInput],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-vg-indirect-draw-refs"),
        contents: bytemuck::cast_slice(draw_refs),
        usage: wgpu::BufferUsages::STORAGE,
    })
}
