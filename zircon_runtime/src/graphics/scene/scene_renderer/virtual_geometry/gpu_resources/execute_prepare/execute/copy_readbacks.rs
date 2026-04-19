use super::super::super::buffer_helpers::buffer_size_for_words;
use super::virtual_geometry_prepare_execution_buffers::VirtualGeometryPrepareExecutionBuffers;
use super::virtual_geometry_prepare_execution_inputs::VirtualGeometryPrepareExecutionInputs;

pub(super) fn copy_readbacks(
    encoder: &mut wgpu::CommandEncoder,
    buffers: &VirtualGeometryPrepareExecutionBuffers,
    inputs: &VirtualGeometryPrepareExecutionInputs,
) {
    encoder.copy_buffer_to_buffer(
        &buffers.completed_buffer,
        0,
        &buffers.completed_readback,
        0,
        buffer_size_for_words(inputs.completed_word_count.max(1)),
    );
    encoder.copy_buffer_to_buffer(
        &buffers.page_table_buffer,
        0,
        &buffers.page_table_readback,
        0,
        buffer_size_for_words(inputs.page_table_word_count),
    );
}
