use super::{
    ibl_bake_wgpu_command_plan_for_runtime_kernel, IblBakeWgpuCommandPlan, IBL_BAKE_CUBE_FACE_COUNT,
};
use crate::core::framework::render::{IblBakeArtifactDescriptor, IblBakeArtifactRequest};
use crate::graphics::scene::scene_renderer::environment::ibl_bake_graph_plan::ibl_bake_pmrem_dispatch_groups_for_face_range;
use crate::graphics::scene::scene_renderer::environment::ibl_bake_shader_plan::ibl_bake_pmrem_kernel_plan;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_time_slice::RealtimeIblPrefilterDispatchSlice;

pub(in crate::graphics::scene::scene_renderer) fn ibl_bake_wgpu_prefilter_command_for_slice(
    request: &IblBakeArtifactRequest,
    slice: RealtimeIblPrefilterDispatchSlice,
) -> Option<IblBakeWgpuCommandPlan> {
    let dispatch_groups = ibl_bake_pmrem_dispatch_groups_for_face_range(
        request.pmrem_face_size(),
        request.pmrem_mip_count(),
        u32::from(slice.mip_level),
        u32::from(slice.first_face),
        u32::from(slice.face_count),
    )?;
    let mut command = ibl_bake_wgpu_command_plan_for_runtime_kernel(
        request,
        IblBakeArtifactDescriptor::current_for_runtime_cache_request(request),
        ibl_bake_pmrem_kernel_plan(request, u32::from(slice.mip_level)),
    );
    command.params.words[5] = u32::from(slice.first_face);
    let writes_all_faces = command.params.words[7] == 1.0_f32.to_bits()
        && slice.first_face == 0
        && slice.face_count == IBL_BAKE_CUBE_FACE_COUNT as u8;
    if !writes_all_faces {
        command.params.words[7] = 0;
    }
    command.dispatch_groups = dispatch_groups;
    Some(command)
}

#[cfg(test)]
mod tests;
