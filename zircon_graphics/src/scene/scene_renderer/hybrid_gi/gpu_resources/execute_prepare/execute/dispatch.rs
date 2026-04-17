use super::super::super::hybrid_gi_gpu_resources::HybridGiGpuResources;
use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;

const HYBRID_GI_COMPLETION_WORKGROUP_SIZE: u32 = 64;

pub(super) fn dispatch(
    resources: &HybridGiGpuResources,
    encoder: &mut wgpu::CommandEncoder,
    bind_group: &wgpu::BindGroup,
    inputs: &HybridGiPrepareExecutionInputs,
) {
    let dispatch_count = inputs
        .resident_probe_inputs
        .len()
        .max(inputs.pending_probe_inputs.len())
        .max(inputs.trace_region_inputs.len());
    if dispatch_count == 0 {
        return;
    }

    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("HybridGiCompletionPass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(&resources.pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.dispatch_workgroups(
        (dispatch_count as u32).div_ceil(HYBRID_GI_COMPLETION_WORKGROUP_SIZE),
        1,
        1,
    );
}
