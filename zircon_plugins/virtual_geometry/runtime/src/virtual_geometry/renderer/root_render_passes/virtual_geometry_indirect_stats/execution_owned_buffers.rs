use std::sync::Arc;

use zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionDraw;

pub(super) fn build_execution_submission_buffer(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    indirect_execution_draws: &[&RenderVirtualGeometryExecutionDraw],
    shared_submission_buffer: Option<&Arc<wgpu::Buffer>>,
) -> Option<Arc<wgpu::Buffer>> {
    let record_stride_bytes = std::mem::size_of::<u32>() as u64;

    let shared_submission_buffer = shared_submission_buffer?;
    if indirect_execution_draws.is_empty() {
        return None;
    }

    let buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-vg-indirect-execution-submission-tokens"),
        size: (indirect_execution_draws.len() as u64) * record_stride_bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    }));

    for (execution_index, draw) in indirect_execution_draws.iter().enumerate() {
        let draw_ref_index = draw.execution_draw_ref_index as u64;
        encoder.copy_buffer_to_buffer(
            shared_submission_buffer,
            draw_ref_index * record_stride_bytes,
            &buffer,
            (execution_index as u64) * record_stride_bytes,
            record_stride_bytes,
        );
    }

    Some(buffer)
}

pub(super) fn build_execution_authority_buffer(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    indirect_execution_draws: &[&RenderVirtualGeometryExecutionDraw],
    shared_authority_buffer: Option<&Arc<wgpu::Buffer>>,
) -> Option<Arc<wgpu::Buffer>> {
    const AUTHORITY_RECORD_WORD_COUNT: u64 = 15;
    let record_stride_bytes = (std::mem::size_of::<u32>() as u64) * AUTHORITY_RECORD_WORD_COUNT;

    let shared_authority_buffer = shared_authority_buffer?;
    if indirect_execution_draws.is_empty() {
        return None;
    }

    let buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-vg-indirect-execution-authority-records"),
        size: (indirect_execution_draws.len() as u64) * record_stride_bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    }));

    for (execution_index, draw) in indirect_execution_draws.iter().enumerate() {
        let draw_ref_index = draw.execution_draw_ref_index as u64;
        encoder.copy_buffer_to_buffer(
            shared_authority_buffer,
            draw_ref_index * record_stride_bytes,
            &buffer,
            (execution_index as u64) * record_stride_bytes,
            record_stride_bytes,
        );
    }

    Some(buffer)
}
