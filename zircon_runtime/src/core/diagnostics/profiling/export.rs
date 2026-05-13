use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use zircon_runtime_interface::{
    HotspotReport, ProfileSnapshot, PROFILE_HOTSPOTS_FILE, PROFILE_SUMMARY_FILE,
    PROFILE_TIMELINE_NATIVE_FILE, PROFILE_TIMELINE_PERFETTO_FILE,
};

use super::analyze_hotspots;

#[derive(Clone, Debug)]
pub struct ProfileExportReport {
    pub snapshot: ProfileSnapshot,
    pub hotspots: HotspotReport,
    pub export_dir: String,
    pub files: Vec<String>,
}

pub fn export_snapshot(snapshot: &ProfileSnapshot) -> Result<ProfileExportReport, String> {
    let hotspots = analyze_hotspots(snapshot);
    let export_dir =
        PathBuf::from(&snapshot.output_root).join(sanitize_session_id(&snapshot.session_id));
    fs::create_dir_all(&export_dir).map_err(|error| error.to_string())?;

    let mut files = Vec::new();
    write_json(&export_dir, PROFILE_TIMELINE_NATIVE_FILE, snapshot)?;
    files.push(PROFILE_TIMELINE_NATIVE_FILE.to_string());
    write_json(
        &export_dir,
        PROFILE_TIMELINE_PERFETTO_FILE,
        &perfetto_trace(snapshot),
    )?;
    files.push(PROFILE_TIMELINE_PERFETTO_FILE.to_string());
    write_json(&export_dir, PROFILE_HOTSPOTS_FILE, &hotspots)?;
    files.push(PROFILE_HOTSPOTS_FILE.to_string());
    fs::write(
        export_dir.join(PROFILE_SUMMARY_FILE),
        summary_markdown(snapshot, &hotspots),
    )
    .map_err(|error| error.to_string())?;
    files.push(PROFILE_SUMMARY_FILE.to_string());

    Ok(ProfileExportReport {
        snapshot: snapshot.clone(),
        hotspots,
        export_dir: export_dir.to_string_lossy().into_owned(),
        files,
    })
}

fn write_json<T: Serialize>(dir: &std::path::Path, name: &str, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    fs::write(dir.join(name), bytes).map_err(|error| error.to_string())
}

fn sanitize_session_id(session_id: &str) -> String {
    session_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
}

#[derive(Serialize)]
struct PerfettoTrace {
    #[serde(rename = "traceEvents")]
    trace_events: Vec<PerfettoEvent>,
}

#[derive(Serialize)]
struct PerfettoEvent {
    name: String,
    cat: String,
    ph: &'static str,
    ts: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    dur: Option<u64>,
    pid: u32,
    tid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<serde_json::Value>,
}

fn perfetto_trace(snapshot: &ProfileSnapshot) -> PerfettoTrace {
    let mut events = Vec::new();
    for frame in &snapshot.frames {
        events.push(PerfettoEvent {
            name: frame.name.clone(),
            cat: "frame".to_string(),
            ph: "X",
            ts: frame.start_us,
            dur: Some(frame.duration_us),
            pid: 1,
            tid: frame.stream.clone(),
            id: None,
            args: Some(serde_json::json!({
                "frame_index": frame.frame_index,
                "over_budget": frame.over_budget,
            })),
        });
    }
    for span in &snapshot.spans {
        events.push(PerfettoEvent {
            name: span.name.clone(),
            cat: span.category.clone(),
            ph: "X",
            ts: span.start_us,
            dur: Some(span.duration_us),
            pid: 1,
            tid: span.stream.clone(),
            id: None,
            args: Some(serde_json::json!({
                "path": span.path,
                "frame_index": span.frame_index,
                "depth": span.depth,
            })),
        });
    }
    for counter in &snapshot.counters {
        events.push(PerfettoEvent {
            name: counter.name.clone(),
            cat: "counter".to_string(),
            ph: "C",
            ts: counter.timestamp_us,
            dur: None,
            pid: 1,
            tid: counter.stream.clone(),
            id: None,
            args: Some(serde_json::json!({ "value": counter.value })),
        });
    }
    PerfettoTrace {
        trace_events: events,
    }
}

fn summary_markdown(snapshot: &ProfileSnapshot, hotspots: &HotspotReport) -> String {
    let over_budget = snapshot
        .frames
        .iter()
        .filter(|frame| frame.over_budget)
        .count();
    let mut summary = format!(
        "# Zircon Profile Summary\n\n- Session: `{}`\n- Frames: {}\n- Spans: {}\n- Counters: {}\n- Frame budget: {:.2} ms\n- Over-budget frames: {}\n\n## Top Hotspots\n",
        snapshot.session_id,
        snapshot.frames.len(),
        snapshot.spans.len(),
        snapshot.counters.len(),
        snapshot.frame_budget_ms,
        over_budget
    );
    for entry in hotspots.hotspots.iter().take(10) {
        summary.push_str(&format!(
            "- `{}` total {:.2} ms, avg {:.2} ms, p95 {:.2} ms, count {}\n",
            entry.path,
            entry.total_us as f64 / 1_000.0,
            entry.avg_us as f64 / 1_000.0,
            entry.p95_us as f64 / 1_000.0,
            entry.count
        ));
    }
    if !hotspots.hints.is_empty() {
        summary.push_str("\n## Hints\n");
        for hint in &hotspots.hints {
            summary.push_str(&format!("- {hint}\n"));
        }
    }
    summary
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{ProfileFrameSnapshot, ProfileSnapshot, ProfileSpanSnapshot};

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

        let report = super::export_snapshot(&snapshot).expect("export profile snapshot");

        assert!(report.export_dir.ends_with("session_with_separators"));
        assert!(report.files.contains(&"timeline.zrtrace.json".to_string()));
        assert!(report.files.contains(&"timeline.perfetto.json".to_string()));
        assert!(report.files.contains(&"hotspots.json".to_string()));
        assert!(report.files.contains(&"summary.md".to_string()));
        assert!(std::path::Path::new(&report.export_dir)
            .join("summary.md")
            .exists());

        let _ = std::fs::remove_dir_all(output_root);
    }
}
