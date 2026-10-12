use super::super::super::virtual_geometry_gpu_resources::VirtualGeometryGpuResources;
use super::virtual_geometry_prepare_execution_inputs::VirtualGeometryPrepareExecutionInputs;

pub(super) fn dispatch(
    resources: &VirtualGeometryGpuResources,
    encoder: &mut wgpu::CommandEncoder,
    bind_group: &wgpu::BindGroup,
    inputs: &VirtualGeometryPrepareExecutionInputs,
) {
    if inputs.pending_requests.is_empty() {
        return;
    }

    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("VirtualGeometryUploaderPass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(&resources.pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.dispatch_workgroups(1, 1, 1);
}
