use super::{
    ibl_bake_wgpu_command_plan_for_request, IblBakeComputeKernelKind, IblBakeWgpuCommandPlan,
    IBL_BAKE_CUBE_FACE_COUNT,
};
use crate::core::framework::render::IblBakeArtifactRequest;
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_time_slice::RealtimeIblPrefilterDispatchSlice;

pub(in crate::graphics::scene::scene_renderer) fn ibl_bake_wgpu_prefilter_command_for_slice(
    request: &IblBakeArtifactRequest,
    slice: RealtimeIblPrefilterDispatchSlice,
) -> Option<IblBakeWgpuCommandPlan> {
    let end_face = u32::from(slice.first_face).checked_add(u32::from(slice.face_count))?;
    if slice.face_count == 0
        || end_face > IBL_BAKE_CUBE_FACE_COUNT
        || u32::from(slice.mip_level) >= request.pmrem_mip_count()
    {
        return None;
    }
    let mut command = ibl_bake_wgpu_command_plan_for_request(request)
        .commands
        .into_iter()
        .find(|command| {
            command.kind
                == IblBakeComputeKernelKind::Pmrem {
                    mip_level: u32::from(slice.mip_level),
                }
        })?;
    command.params.words[5] = u32::from(slice.first_face);
    command.dispatch_groups[2] = u32::from(slice.face_count);
    command.readback_copies.clear();
    Some(command)
}

#[cfg(test)]
mod tests;
