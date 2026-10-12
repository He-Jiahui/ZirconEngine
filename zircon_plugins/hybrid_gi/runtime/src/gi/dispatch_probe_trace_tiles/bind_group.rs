use wgpu::util::DeviceExt;

use super::*;

const FALLBACK_VOXEL_CELL_LOOKUP_WORD_COUNT: usize = 8 * (1 + 64);

#[allow(clippy::too_many_arguments)]
pub(super) fn create_probe_trace_tile_dispatch_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    params_buffer: &wgpu::Buffer,
    resident_probe_buffer: &wgpu::Buffer,
    pending_probe_buffer: &wgpu::Buffer,
    probe_trace_tile_buffer: &wgpu::Buffer,
    trace_lighting_buffer: &wgpu::Buffer,
    surface_cache_atlas_view: &wgpu::TextureView,
    surface_cache_depth_view: &wgpu::TextureView,
    scene_prepare_descriptor_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    let voxel_cell_lookup_buffer = create_fallback_voxel_cell_lookup_buffer(device);
    create_probe_trace_tile_dispatch_bind_group_with_voxel_lookup(
        device,
        bind_group_layout,
        params_buffer,
        resident_probe_buffer,
        pending_probe_buffer,
        probe_trace_tile_buffer,
        trace_lighting_buffer,
        surface_cache_atlas_view,
        surface_cache_depth_view,
        scene_prepare_descriptor_buffer,
        &voxel_cell_lookup_buffer,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn create_probe_trace_tile_dispatch_bind_group_with_voxel_lookup(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    params_buffer: &wgpu::Buffer,
    resident_probe_buffer: &wgpu::Buffer,
    pending_probe_buffer: &wgpu::Buffer,
    probe_trace_tile_buffer: &wgpu::Buffer,
    trace_lighting_buffer: &wgpu::Buffer,
    surface_cache_atlas_view: &wgpu::TextureView,
    surface_cache_depth_view: &wgpu::TextureView,
    scene_prepare_descriptor_buffer: &wgpu::Buffer,
    voxel_cell_lookup_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    let empty_global_sdf_page_table_buffer = create_zeroed_global_sdf_trace_buffer(
        device,
        "zircon-hybrid-gi-probe-trace-empty-global-sdf-page-table",
    );
    let empty_global_sdf_atlas_buffer = create_zeroed_global_sdf_trace_buffer(
        device,
        "zircon-hybrid-gi-probe-trace-empty-global-sdf-atlas",
    );
    let trace_diagnostic_buffer = create_fallback_trace_diagnostic_buffer(device);
    create_probe_trace_tile_dispatch_bind_group_from_buffers_with_diagnostics_and_voxel_lookup(
        device,
        bind_group_layout,
        params_buffer,
        resident_probe_buffer,
        pending_probe_buffer,
        probe_trace_tile_buffer,
        trace_lighting_buffer,
        &trace_diagnostic_buffer,
        surface_cache_atlas_view,
        surface_cache_depth_view,
        scene_prepare_descriptor_buffer,
        voxel_cell_lookup_buffer,
        &empty_global_sdf_page_table_buffer,
        &empty_global_sdf_atlas_buffer,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn create_probe_trace_tile_dispatch_bind_group_with_global_sdf(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    params_buffer: &wgpu::Buffer,
    resident_probe_buffer: &wgpu::Buffer,
    pending_probe_buffer: &wgpu::Buffer,
    probe_trace_tile_buffer: &wgpu::Buffer,
    trace_lighting_buffer: &wgpu::Buffer,
    trace_diagnostic_buffer: &wgpu::Buffer,
    surface_cache_atlas_view: &wgpu::TextureView,
    surface_cache_depth_view: &wgpu::TextureView,
    scene_prepare_descriptor_buffer: &wgpu::Buffer,
    voxel_cell_lookup_buffer: &wgpu::Buffer,
    global_sdf_bindings: &GlobalSdfGpuTraceBindings,
) -> wgpu::BindGroup {
    create_probe_trace_tile_dispatch_bind_group_from_buffers_with_diagnostics_and_voxel_lookup(
        device,
        bind_group_layout,
        params_buffer,
        resident_probe_buffer,
        pending_probe_buffer,
        probe_trace_tile_buffer,
        trace_lighting_buffer,
        trace_diagnostic_buffer,
        surface_cache_atlas_view,
        surface_cache_depth_view,
        scene_prepare_descriptor_buffer,
        voxel_cell_lookup_buffer,
        &global_sdf_bindings.page_table_buffer,
        &global_sdf_bindings.atlas_buffer,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn create_probe_trace_tile_dispatch_bind_group_from_buffers_with_diagnostics(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    params_buffer: &wgpu::Buffer,
    resident_probe_buffer: &wgpu::Buffer,
    pending_probe_buffer: &wgpu::Buffer,
    probe_trace_tile_buffer: &wgpu::Buffer,
    trace_lighting_buffer: &wgpu::Buffer,
    trace_diagnostic_buffer: &wgpu::Buffer,
    surface_cache_atlas_view: &wgpu::TextureView,
    surface_cache_depth_view: &wgpu::TextureView,
    scene_prepare_descriptor_buffer: &wgpu::Buffer,
    global_sdf_page_table_buffer: &wgpu::Buffer,
    global_sdf_atlas_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    let voxel_cell_lookup_buffer = create_fallback_voxel_cell_lookup_buffer(device);
    create_probe_trace_tile_dispatch_bind_group_from_buffers_with_diagnostics_and_voxel_lookup(
        device,
        bind_group_layout,
        params_buffer,
        resident_probe_buffer,
        pending_probe_buffer,
        probe_trace_tile_buffer,
        trace_lighting_buffer,
        trace_diagnostic_buffer,
        surface_cache_atlas_view,
        surface_cache_depth_view,
        scene_prepare_descriptor_buffer,
        &voxel_cell_lookup_buffer,
        global_sdf_page_table_buffer,
        global_sdf_atlas_buffer,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn create_probe_trace_tile_dispatch_bind_group_from_buffers_with_diagnostics_and_voxel_lookup(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    params_buffer: &wgpu::Buffer,
    resident_probe_buffer: &wgpu::Buffer,
    pending_probe_buffer: &wgpu::Buffer,
    probe_trace_tile_buffer: &wgpu::Buffer,
    trace_lighting_buffer: &wgpu::Buffer,
    trace_diagnostic_buffer: &wgpu::Buffer,
    surface_cache_atlas_view: &wgpu::TextureView,
    surface_cache_depth_view: &wgpu::TextureView,
    scene_prepare_descriptor_buffer: &wgpu::Buffer,
    voxel_cell_lookup_buffer: &wgpu::Buffer,
    global_sdf_page_table_buffer: &wgpu::Buffer,
    global_sdf_atlas_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hybrid-gi-probe-trace-tile-dispatch-bind-group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: resident_probe_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: pending_probe_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: probe_trace_tile_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: trace_lighting_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(surface_cache_atlas_view),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::TextureView(surface_cache_depth_view),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: scene_prepare_descriptor_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 8,
                resource: global_sdf_page_table_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 9,
                resource: global_sdf_atlas_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 10,
                resource: trace_diagnostic_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 11,
                resource: voxel_cell_lookup_buffer.as_entire_binding(),
            },
        ],
    })
}

fn create_fallback_trace_diagnostic_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-hybrid-gi-probe-trace-fallback-diagnostics"),
        contents: bytemuck::cast_slice(&[0_u32; FALLBACK_TRACE_DIAGNOSTIC_WORD_COUNT]),
        usage: wgpu::BufferUsages::STORAGE,
    })
}

fn create_fallback_voxel_cell_lookup_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-hybrid-gi-probe-trace-fallback-voxel-cell-lookup"),
        contents: bytemuck::cast_slice(&[u32::MAX; FALLBACK_VOXEL_CELL_LOOKUP_WORD_COUNT]),
        usage: wgpu::BufferUsages::STORAGE,
    })
}

fn create_zeroed_global_sdf_trace_buffer(
    device: &wgpu::Device,
    label: &'static str,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::bytes_of(&0_u32),
        usage: wgpu::BufferUsages::STORAGE,
    })
}
