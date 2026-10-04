use super::profile_session_basename;
use zircon_runtime_interface::{
    ProfileCounterSnapshot, ProfileFrameSnapshot, ProfileSnapshot, ProfileSpanSnapshot,
};

#[test]
fn profile_session_id_sanitization_always_produces_a_child_basename() {
    let separated = profile_session_basename("session/with:separators");
    assert!(separated.starts_with("session_with_separators-"));
    for (session_id, expected_prefix) in [
        ("CON", "session_CON-"),
        ("con.txt", "session_con.txt-"),
        ("NUL", "session_NUL-"),
        ("COM1", "session_COM1-"),
        ("LPT9.log", "session_LPT9.log-"),
    ] {
        assert!(
            profile_session_basename(session_id).starts_with(expected_prefix),
            "session_id={session_id:?}"
        );
    }

    for session_id in ["", ".", "..", "..."] {
        let sanitized = profile_session_basename(session_id);
        assert!(!sanitized.is_empty(), "session_id={session_id:?}");
        assert_ne!(sanitized, ".", "session_id={session_id:?}");
        assert_ne!(sanitized, "..", "session_id={session_id:?}");
        assert!(!sanitized.ends_with('.'), "session_id={session_id:?}");
        assert_eq!(
            std::path::Path::new(&sanitized).components().count(),
            1,
            "session_id={session_id:?}"
        );
    }
}

#[test]
fn profile_session_id_sanitization_is_collision_resistant_and_bounded() {
    let colon = profile_session_basename("a:b");
    let question = profile_session_basename("a?b");
    let already_safe = profile_session_basename("a_b");

    assert_ne!(colon, question);
    assert_ne!(colon, already_safe);
    assert_ne!(question, already_safe);
    for sanitized in [
        colon,
        question,
        already_safe,
        profile_session_basename(&"a".repeat(1_024)),
    ] {
        assert!(
            sanitized.len()
                <= zircon_runtime_interface::profiling::PROFILE_SESSION_BASENAME_MAX_BYTES,
            "sanitized basename is too long: {} bytes",
            sanitized.len()
        );
        assert_eq!(std::path::Path::new(&sanitized).components().count(), 1);
    }
}

#[test]
fn perfetto_trace_contains_complete_event_spans() {
    let mut snapshot = ProfileSnapshot::default();
    snapshot.spans.push(ProfileSpanSnapshot {
        id: 1,
        parent_id: None,
        frame_index: Some(0),
        stream: "runtime".to_string(),
        category: "render".to_string(),
        name: "submit".to_string(),
        path: "runtime/render:submit".to_string(),
        start_us: 7,
        duration_us: 11,
        depth: 0,
    });

    let trace = super::perfetto_trace(&snapshot);

    assert_eq!(trace.trace_events.len(), 1);
    assert_eq!(trace.trace_events[0].ph, "X");
    assert_eq!(trace.trace_events[0].dur, Some(11));
}

#[test]
fn optimization_batch_20260826f_runtime03_perfetto_projection_preserves_json_shape() {
    let mut snapshot = ProfileSnapshot::default();
    snapshot.frames.push(ProfileFrameSnapshot {
        stream: "runtime".to_string(),
        name: "frame".to_string(),
        frame_index: 3,
        start_us: 5,
        duration_us: 7,
        budget_ms: 16.67,
        over_budget: true,
    });
    snapshot.spans.push(ProfileSpanSnapshot {
        id: 9,
        parent_id: None,
        frame_index: Some(3),
        stream: "render".to_string(),
        category: "submit".to_string(),
        name: "present".to_string(),
        path: "runtime/render:present".to_string(),
        start_us: 11,
        duration_us: 13,
        depth: 2,
    });
    snapshot.counters.push(runtime_counter("draw_calls", 17.0));

    let value = serde_json::to_value(super::perfetto_trace(&snapshot))
        .expect("serialize borrowed Perfetto projection");
    let events = value["traceEvents"].as_array().expect("traceEvents array");

    assert_eq!(events.len(), 3);
    assert_eq!(events[0]["name"], "frame");
    assert_eq!(events[0]["cat"], "frame");
    assert_eq!(events[0]["args"]["frame_index"], 3);
    assert_eq!(events[0]["args"]["over_budget"], true);
    assert_eq!(events[1]["name"], "present");
    assert_eq!(events[1]["cat"], "submit");
    assert_eq!(events[1]["args"]["path"], "runtime/render:present");
    assert_eq!(events[1]["args"]["depth"], 2);
    assert_eq!(events[2]["ph"], "C");
    assert_eq!(events[2]["args"]["value"], 17.0);
}

#[test]
fn optimization_batch_20260826f_runtime03_perfetto_projection_borrows_event_text() {
    let source = include_str!("../export.rs");
    let projection = source
        .split("fn perfetto_trace")
        .nth(1)
        .expect("Perfetto projection")
        .split("fn summary_markdown")
        .next()
        .expect("bounded Perfetto projection");

    assert!(source.contains("struct PerfettoEvent<'a>"));
    assert!(source.contains("enum PerfettoArgs<'a>"));
    assert!(projection.contains("Vec::with_capacity"));
    assert!(!projection.contains(".clone()"));
    assert!(!projection.contains("serde_json::json!"));
    assert!(!projection.contains(".to_string()"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260826f_runtime03_perfetto_projection_performance_evidence() {
    use std::hint::black_box;
    use std::time::Instant;

    struct LegacyEvent {
        name: String,
        category: String,
        stream: String,
        args: serde_json::Value,
    }

    let mut snapshot = ProfileSnapshot::default();
    for index in 0..32_768_u64 {
        snapshot.spans.push(ProfileSpanSnapshot {
            id: index,
            parent_id: None,
            frame_index: Some(index / 64),
            stream: "runtime".to_string(),
            category: "render".to_string(),
            name: "submit".to_string(),
            path: "runtime/render:submit".to_string(),
            start_us: index,
            duration_us: 3,
            depth: 1,
        });
    }

    let mut legacy_samples = Vec::with_capacity(17);
    let mut borrowed_samples = Vec::with_capacity(17);
    for _ in 0..17 {
        let started = Instant::now();
        let events = snapshot
            .spans
            .iter()
            .map(|span| LegacyEvent {
                name: span.name.clone(),
                category: span.category.clone(),
                stream: span.stream.clone(),
                args: serde_json::json!({
                    "path": span.path,
                    "frame_index": span.frame_index,
                    "depth": span.depth,
                }),
            })
            .collect::<Vec<_>>();
        black_box(events);
        legacy_samples.push(started.elapsed().as_nanos());

        let started = Instant::now();
        let trace = super::perfetto_trace(black_box(&snapshot));
        black_box(trace.trace_events.len());
        borrowed_samples.push(started.elapsed().as_nanos());
    }

    legacy_samples.sort_unstable();
    borrowed_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let borrowed_p95 = borrowed_samples[16];
    println!(
        "RUNTIME03_PERFETTO_BORROWED_EVENT_PROJECTION_BENCH_V1 events={} legacy_p95_ns={} borrowed_p95_ns={} legacy_owned_event_fields={} borrowed_owned_event_fields=0 target_ratio_bp=6000",
        snapshot.spans.len(),
        legacy_p95,
        borrowed_p95,
        snapshot.spans.len() * 4,
    );
    assert!(
        borrowed_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
        "borrowed Perfetto projection P95 {borrowed_p95} ns exceeded 60% of legacy {legacy_p95} ns"
    );
}

#[test]
fn export_snapshot_writes_expected_profile_artifacts() {
    let output_root =
        std::env::temp_dir().join(format!("zircon-profile-export-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&output_root);
    let mut snapshot = ProfileSnapshot {
        session_id: "session/with:separators".to_string(),
        output_root: output_root.to_string_lossy().into_owned(),
        ..ProfileSnapshot::default()
    };
    snapshot.frames.push(ProfileFrameSnapshot {
        stream: "runtime".to_string(),
        name: "frame".to_string(),
        frame_index: 0,
        start_us: 1,
        duration_us: 2,
        budget_ms: 16.67,
        over_budget: false,
    });

    let report = super::export_snapshot(&snapshot, true).expect("export profile snapshot");

    assert!(report
        .export_dir
        .ends_with(&profile_session_basename("session/with:separators")));
    assert!(report.files.contains(&"timeline.zrtrace.json".to_string()));
    assert!(report.files.contains(&"timeline.perfetto.json".to_string()));
    assert!(report.files.contains(&"hotspots.json".to_string()));
    assert!(report.files.contains(&"counter_hotspots.json".to_string()));
    assert!(report.files.contains(&"ui_hotspots.json".to_string()));
    assert!(report.files.contains(&"summary.md".to_string()));
    assert!(std::path::Path::new(&report.export_dir)
        .join("counter_hotspots.json")
        .exists());
    assert!(std::path::Path::new(&report.export_dir)
        .join("ui_hotspots.json")
        .exists());
    assert!(std::path::Path::new(&report.export_dir)
        .join("summary.md")
        .exists());

    let _ = std::fs::remove_dir_all(output_root);
}

#[test]
fn export_snapshot_skips_perfetto_when_not_requested() {
    let output_root = std::env::temp_dir().join(format!(
        "zircon-profile-export-no-perfetto-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&output_root);
    let snapshot = ProfileSnapshot {
        session_id: "no-perfetto".to_string(),
        output_root: output_root.to_string_lossy().into_owned(),
        ..ProfileSnapshot::default()
    };

    let report = super::export_snapshot(&snapshot, false).expect("export profile snapshot");

    assert!(!report.files.contains(&"timeline.perfetto.json".to_string()));
    assert!(!std::path::Path::new(&report.export_dir)
        .join("timeline.perfetto.json")
        .exists());

    let _ = std::fs::remove_dir_all(output_root);
}

#[test]
fn export_snapshot_removes_perfetto_from_a_reused_session_directory() {
    let output_root = std::env::temp_dir().join(format!(
        "zircon-profile-export-perfetto-reuse-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&output_root);
    let snapshot = ProfileSnapshot {
        session_id: "reused-session".to_string(),
        output_root: output_root.to_string_lossy().into_owned(),
        ..ProfileSnapshot::default()
    };

    let first = super::export_snapshot(&snapshot, true).expect("export with Perfetto");
    let perfetto_path = std::path::Path::new(&first.export_dir).join("timeline.perfetto.json");
    assert!(perfetto_path.exists());

    let second = super::export_snapshot(&snapshot, false).expect("export without Perfetto");
    assert_eq!(second.export_dir, first.export_dir);
    assert!(!second.files.contains(&"timeline.perfetto.json".to_string()));
    assert!(
        !perfetto_path.exists(),
        "stale Perfetto output must be removed"
    );

    let _ = std::fs::remove_dir_all(output_root);
}

#[test]
fn export_snapshot_reports_failure_to_remove_stale_perfetto() {
    let output_root = std::env::temp_dir().join(format!(
        "zircon-profile-export-perfetto-remove-error-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&output_root);
    let snapshot = ProfileSnapshot {
        session_id: "blocked-perfetto-removal".to_string(),
        output_root: output_root.to_string_lossy().into_owned(),
        ..ProfileSnapshot::default()
    };
    let first = super::export_snapshot(&snapshot, false).expect("fresh export");
    std::fs::create_dir(std::path::Path::new(&first.export_dir).join("timeline.perfetto.json"))
        .expect("block stale Perfetto removal with a directory");

    let error = super::export_snapshot(&snapshot, false).unwrap_err();
    assert!(matches!(
        &error,
        super::ProfileExportError::RemoveFile { .. }
    ));
    assert!(error.to_string().contains("remove profile export file"));

    let _ = std::fs::remove_dir_all(output_root);
}

#[test]
fn export_snapshot_reports_typed_directory_error_source() {
    let output_root = std::env::temp_dir().join(format!(
        "zircon-profile-export-file-parent-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&output_root);
    let _ = std::fs::remove_file(&output_root);
    std::fs::write(&output_root, b"not a directory").expect("write profile export blocker");
    let snapshot = ProfileSnapshot {
        session_id: "typed-error".to_string(),
        output_root: output_root.to_string_lossy().into_owned(),
        ..ProfileSnapshot::default()
    };

    let error = super::export_snapshot(&snapshot, false).unwrap_err();

    assert!(matches!(
        error,
        super::ProfileExportError::CreateExportDirectory { .. }
    ));
    assert!(error
        .to_string()
        .contains("create profile export directory"));

    let _ = std::fs::remove_file(output_root);
}

#[test]
fn summary_lists_first_fix_candidates_from_ui_alerts() {
    let mut snapshot = ProfileSnapshot {
        session_id: "first-fix-test".to_string(),
        ..ProfileSnapshot::default()
    };
    snapshot
        .counters
        .push(counter("ui.idle_hover.redraw_region", 1.0));
    snapshot
        .counters
        .push(counter("ui.idle_hover.full_paint_count", 1.0));

    let hotspots = crate::core::diagnostics::profiling::analyze_hotspots(&snapshot);
    let counter_hotspots = crate::core::diagnostics::profiling::analyze_counter_hotspots(&snapshot);
    let ui_hotspots = crate::core::diagnostics::profiling::analyze_ui_hotspots(&snapshot);
    let summary = super::summary_markdown(&snapshot, &hotspots, &counter_hotspots, &ui_hotspots);

    assert!(summary.contains("## First Fix Candidates"));
    assert!(summary.contains("region_request_repainted_full_frame"));
}

#[test]
fn summary_lists_counter_hotspots_when_no_ui_alert_or_span_hotspot() {
    let mut snapshot = ProfileSnapshot {
        session_id: "counter-first-fix-test".to_string(),
        ..ProfileSnapshot::default()
    };
    snapshot
        .counters
        .push(runtime_counter("asset.worker.frame_completed", 3.0));

    let hotspots = crate::core::diagnostics::profiling::analyze_hotspots(&snapshot);
    let counter_hotspots = crate::core::diagnostics::profiling::analyze_counter_hotspots(&snapshot);
    let ui_hotspots = crate::core::diagnostics::profiling::analyze_ui_hotspots(&snapshot);
    let summary = super::summary_markdown(&snapshot, &hotspots, &counter_hotspots, &ui_hotspots);

    assert!(summary.contains("## Counter Hotspots"));
    assert!(summary.contains("runtime/counter:asset.worker.frame_completed"));
    assert!(summary.contains("Counter `runtime/counter:asset.worker.frame_completed`"));
}

fn counter(name: &str, value: f64) -> ProfileCounterSnapshot {
    ProfileCounterSnapshot {
        stream: "editor".to_string(),
        name: name.to_string(),
        value,
        timestamp_us: 0,
        frame_index: None,
    }
}

fn runtime_counter(name: &str, value: f64) -> ProfileCounterSnapshot {
    ProfileCounterSnapshot {
        stream: "runtime".to_string(),
        name: name.to_string(),
        value,
        timestamp_us: 0,
        frame_index: None,
    }
}
