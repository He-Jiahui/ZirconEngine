use crate::core::framework::render::RenderStats;

use super::{DiagnosticStore, record_bytes, record_count};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.volumetric_fog.compute_dispatch_count",
        frame_index,
        stats.last_volumetric_fog_compute_dispatch_count,
        &["render", "volumetric_fog", "compute"],
    );
    record_count(
        store,
        "render.volumetric_fog.compute_dispatch_group_count",
        frame_index,
        stats.last_volumetric_fog_compute_dispatch_group_count,
        &["render", "volumetric_fog", "compute", "workgroup"],
    );
    record_bytes(
        store,
        "render.volumetric_fog.uploaded_bytes",
        frame_index,
        stats.last_volumetric_fog_uploaded_bytes,
        &["render", "volumetric_fog", "upload"],
    );
}

#[cfg(test)]
mod tests {
    use crate::core::diagnostics::DiagnosticStore;
    use crate::core::framework::render::RenderStats;

    use super::record;

    #[test]
    fn render_perf_volumetric_fog_records_dispatch_and_upload_metrics() {
        let stats = RenderStats {
            submitted_frames: 12,
            last_volumetric_fog_compute_dispatch_count: 3,
            last_volumetric_fog_compute_dispatch_group_count: 44_400,
            last_volumetric_fog_uploaded_bytes: 624,
            ..Default::default()
        };
        let mut store = DiagnosticStore::default();

        record(&mut store, &stats);

        let snapshot = store.snapshot();
        for (path, value, unit) in [
            ("render.volumetric_fog.compute_dispatch_count", 3.0, "count"),
            (
                "render.volumetric_fog.compute_dispatch_group_count",
                44_400.0,
                "count",
            ),
            ("render.volumetric_fog.uploaded_bytes", 624.0, "bytes"),
        ] {
            let series = snapshot
                .series
                .iter()
                .find(|series| series.path.as_str() == path)
                .unwrap_or_else(|| panic!("missing diagnostic series `{path}`"));
            assert_eq!(series.current, Some(value));
            assert_eq!(series.unit.as_deref(), Some(unit));
            assert_eq!(series.history[0].frame_index, 12);
        }
    }
}
