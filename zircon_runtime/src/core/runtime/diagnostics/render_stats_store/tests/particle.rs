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
