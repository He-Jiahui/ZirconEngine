use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::submission_record_update::SubmissionRecordUpdate;

pub(super) fn update_particle_stats(
    state: &mut RenderFrameworkState,
    record_update: &SubmissionRecordUpdate,
) {
    let stats = record_update.particle_stats();
    state.stats.last_particle_gpu_alive_count = stats.gpu_alive_count();
    state.stats.last_particle_gpu_spawned_total = stats.gpu_spawned_total();
    state.stats.last_particle_gpu_emitter_readback_count = stats.gpu_emitter_readback_count();
    state.stats.last_particle_gpu_indirect_instance_count = stats.gpu_indirect_instance_count();
}
