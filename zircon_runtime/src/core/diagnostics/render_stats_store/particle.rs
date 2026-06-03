use crate::core::framework::render::RenderStats;

use super::{record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.particle.gpu.alive_count",
        frame_index,
        stats.last_particle_gpu_alive_count,
        &["render", "particle", "gpu"],
    );
    record_count(
        store,
        "render.particle.gpu.spawned_total",
        frame_index,
        stats.last_particle_gpu_spawned_total,
        &["render", "particle", "gpu"],
    );
    record_count(
        store,
        "render.particle.gpu.emitter_readback_count",
        frame_index,
        stats.last_particle_gpu_emitter_readback_count,
        &["render", "particle", "gpu", "readback"],
    );
    record_count(
        store,
        "render.particle.gpu.indirect_instance_count",
        frame_index,
        stats.last_particle_gpu_indirect_instance_count,
        &["render", "particle", "gpu", "indirect"],
    );
}
