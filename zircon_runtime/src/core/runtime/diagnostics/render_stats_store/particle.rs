use crate::core::framework::render::RenderStats;

use super::{record_count, DiagnosticStore};

/// 将粒子存活、生成和速度流歧义并列暴露，以区分调度规模与速度数据缺口。
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
        "render.particle.velocity.missing_sprite_count",
        frame_index,
        stats.last_particle_velocity_missing_sprite_count,
        &["render", "particle", "velocity", "missing"],
    );
    record_count(
        store,
        "render.particle.velocity.anonymous_stream_ambiguity_count",
        frame_index,
        stats.last_particle_velocity_anonymous_stream_ambiguity_count,
        &["render", "particle", "velocity", "anonymous"],
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

#[cfg(test)]
#[path = "tests/particle.rs"]
mod tests;
