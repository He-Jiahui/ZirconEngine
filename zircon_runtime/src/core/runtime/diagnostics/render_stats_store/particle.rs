use crate::core::framework::render::RenderStats;

use super::{DiagnosticStore, record_count};

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
mod tests {
    use crate::core::framework::render::RenderStats;
    use crate::core::runtime::diagnostics::DiagnosticStore;

    use super::record;

    #[test]
    fn particle_diagnostics_record_anonymous_stream_ambiguity_count() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_particle_velocity_anonymous_stream_ambiguity_count: 2,
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        let snapshot = store.snapshot();
        let series = snapshot
            .series
            .iter()
            .find(|series| {
                series.path.as_str() == "render.particle.velocity.anonymous_stream_ambiguity_count"
            })
            .expect("missing anonymous particle velocity diagnostic series");
        assert_eq!(series.current, Some(2.0));
        assert_eq!(series.unit.as_deref(), Some("count"));
        assert_eq!(
            series.subsystem_tags,
            vec!["anonymous", "particle", "render", "velocity"]
        );
    }
}
