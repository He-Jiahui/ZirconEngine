use super::super::super::buffer_helpers::buffer_size_for_words;
use super::hybrid_gi_prepare_execution_buffers::HybridGiPrepareExecutionBuffers;
use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;

pub(super) fn copy_readbacks(
    encoder: &mut wgpu::CommandEncoder,
    buffers: &HybridGiPrepareExecutionBuffers,
    inputs: &HybridGiPrepareExecutionInputs,
) {
    encoder.copy_buffer_to_buffer(
        &buffers.completed_probe_buffer,
        0,
        &buffers.completed_probe_readback,
        0,
        buffer_size_for_words(inputs.completed_probe_word_count.max(1)),
    );
    encoder.copy_buffer_to_buffer(
        &buffers.completed_trace_buffer,
        0,
        &buffers.completed_trace_readback,
        0,
        buffer_size_for_words(inputs.completed_trace_word_count.max(1)),
    );
    encoder.copy_buffer_to_buffer(
        &buffers.irradiance_buffer,
        0,
        &buffers.irradiance_readback,
        0,
        buffer_size_for_words(inputs.irradiance_word_count.max(1)),
    );
}
