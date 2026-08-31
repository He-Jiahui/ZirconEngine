use std::sync::Arc;

use zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionDraw;

fn visit_coalesced_execution_copy_ranges(
    draw_ref_indices: impl IntoIterator<Item = u32>,
    mut visit: impl FnMut(u64, u64, u64),
) {
    let mut indexed_draw_refs = draw_ref_indices.into_iter().enumerate();
    let Some((first_destination_index, first_draw_ref_index)) = indexed_draw_refs.next() else {
        return;
    };

    let mut source_start = first_draw_ref_index;
    let mut previous_source = first_draw_ref_index;
    let mut destination_start = first_destination_index as u64;
    let mut record_count = 1_u64;
    for (destination_index, draw_ref_index) in indexed_draw_refs {
        if previous_source.checked_add(1) == Some(draw_ref_index) {
            previous_source = draw_ref_index;
            record_count = record_count.saturating_add(1);
            continue;
        }

        visit(source_start.into(), destination_start, record_count);
        source_start = draw_ref_index;
        previous_source = draw_ref_index;
        destination_start = destination_index as u64;
        record_count = 1;
    }
    visit(source_start.into(), destination_start, record_count);
}

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

    visit_coalesced_execution_copy_ranges(
        indirect_execution_draws
            .iter()
            .map(|draw| draw.execution_draw_ref_index),
        |source_record_index, destination_record_index, record_count| {
            encoder.copy_buffer_to_buffer(
                shared_submission_buffer,
                source_record_index * record_stride_bytes,
                &buffer,
                destination_record_index * record_stride_bytes,
                record_count * record_stride_bytes,
            );
        },
    );

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

    visit_coalesced_execution_copy_ranges(
        indirect_execution_draws
            .iter()
            .map(|draw| draw.execution_draw_ref_index),
        |source_record_index, destination_record_index, record_count| {
            encoder.copy_buffer_to_buffer(
                shared_authority_buffer,
                source_record_index * record_stride_bytes,
                &buffer,
                destination_record_index * record_stride_bytes,
                record_count * record_stride_bytes,
            );
        },
    );

    Some(buffer)
}

#[cfg(test)]
mod performance_tests;
