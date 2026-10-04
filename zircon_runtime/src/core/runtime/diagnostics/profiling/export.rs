use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;
use zircon_runtime_interface::profiling::profile_session_basename;
use zircon_runtime_interface::{
    CounterHotspotReport, HotspotReport, ProfileSnapshot, UiHotspotReport,
    PROFILE_COUNTER_HOTSPOTS_FILE, PROFILE_HOTSPOTS_FILE, PROFILE_SUMMARY_FILE,
    PROFILE_TIMELINE_NATIVE_FILE, PROFILE_TIMELINE_PERFETTO_FILE, PROFILE_UI_HOTSPOTS_FILE,
};

use super::{analyze_counter_hotspots, analyze_hotspots, analyze_ui_hotspots};

mod json_writer;
mod session_owner;
#[cfg(test)]
#[path = "export/tests/session_owner_tests.rs"]
mod session_owner_tests;
#[cfg(test)]
#[path = "export/tests/streaming_tests.rs"]
mod streaming_tests;

use json_writer::write_json;

pub type ProfileExportResult<T> = std::result::Result<T, ProfileExportError>;

#[derive(Debug, Error)]
pub enum ProfileExportError {
    #[error("profiling feature is disabled")]
    FeatureDisabled,
    #[error("create profile export directory `{path}` failed: {source}")]
    CreateExportDirectory {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("serialize profile export JSON `{file}` failed: {source}")]
    JsonSerialize {
        file: &'static str,
        #[source]
        source: serde_json::Error,
    },
    #[error("write profile export file `{path}` failed: {source}")]
    WriteFile {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("remove profile export file `{path}` failed: {source}")]
    RemoveFile {
        path: String,
        #[source]
        source: io::Error,
    },
}

/// 一次成功导出的内存摘要和文件清单，供控制命令及性能测试继续定位报告。
#[derive(Clone, Debug)]
pub struct ProfileExportReport {
    pub snapshot: ProfileSnapshot,
    pub hotspots: HotspotReport,
    pub counter_hotspots: CounterHotspotReport,
    pub ui_hotspots: UiHotspotReport,
    pub export_dir: String,
    pub files: Vec<String>,
}

/// 将已冻结的录制快照写到按 session 命名的目录，并生成时间线与热点报告。
/// 同一 session 会覆盖已有文件；中途写入失败可能留下部分文件，成功返回的 files 才是完整清单。
pub fn export_snapshot(
    snapshot: &ProfileSnapshot,
    include_perfetto: bool,
) -> ProfileExportResult<ProfileExportReport> {
    export_snapshot_with_after_native(snapshot, include_perfetto, || {})
}

fn export_snapshot_with_after_native(
    snapshot: &ProfileSnapshot,
    include_perfetto: bool,
    after_native: impl FnOnce(),
) -> ProfileExportResult<ProfileExportReport> {
    let hotspots = analyze_hotspots(snapshot);
    let counter_hotspots = analyze_counter_hotspots(snapshot);
    let ui_hotspots = analyze_ui_hotspots(snapshot);
    let export_dir =
        PathBuf::from(&snapshot.output_root).join(profile_session_basename(&snapshot.session_id));
    fs::create_dir_all(&export_dir).map_err(|source| {
        ProfileExportError::CreateExportDirectory {
            path: path_string(&export_dir),
            source,
        }
    })?;

    // 同一规范目录的整次写出串行执行，避免同一 session 的文件来自交错导出。
    session_owner::with_session_owner(&export_dir, || {
        let mut files = Vec::new();
        write_json(&export_dir, PROFILE_TIMELINE_NATIVE_FILE, snapshot)?;
        files.push(PROFILE_TIMELINE_NATIVE_FILE.to_string());
        after_native();
        if include_perfetto {
            write_json(
                &export_dir,
                PROFILE_TIMELINE_PERFETTO_FILE,
                &perfetto_trace(snapshot),
            )?;
            files.push(PROFILE_TIMELINE_PERFETTO_FILE.to_string());
        } else {
            // Reused session directories must not expose a previous capture's optional trace.
            let perfetto_path = export_dir.join(PROFILE_TIMELINE_PERFETTO_FILE);
            match fs::remove_file(&perfetto_path) {
                Ok(()) => {}
                Err(source) if source.kind() == io::ErrorKind::NotFound => {}
                Err(source) => {
                    return Err(ProfileExportError::RemoveFile {
                        path: path_string(&perfetto_path),
                        source,
                    });
                }
            }
        }
        write_json(&export_dir, PROFILE_HOTSPOTS_FILE, &hotspots)?;
        files.push(PROFILE_HOTSPOTS_FILE.to_string());
        write_json(
            &export_dir,
            PROFILE_COUNTER_HOTSPOTS_FILE,
            &counter_hotspots,
        )?;
        files.push(PROFILE_COUNTER_HOTSPOTS_FILE.to_string());
        write_json(&export_dir, PROFILE_UI_HOTSPOTS_FILE, &ui_hotspots)?;
        files.push(PROFILE_UI_HOTSPOTS_FILE.to_string());
        let summary_path = export_dir.join(PROFILE_SUMMARY_FILE);
        fs::write(
            &summary_path,
            summary_markdown(snapshot, &hotspots, &counter_hotspots, &ui_hotspots),
        )
        .map_err(|source| ProfileExportError::WriteFile {
            path: path_string(&summary_path),
            source,
        })?;
        files.push(PROFILE_SUMMARY_FILE.to_string());

        Ok(ProfileExportReport {
            snapshot: snapshot.clone(),
            hotspots,
            counter_hotspots,
            ui_hotspots,
            export_dir: export_dir.to_string_lossy().into_owned(),
            files,
        })
    })
    .map_err(|source| ProfileExportError::CreateExportDirectory {
        path: path_string(&export_dir),
        source,
    })?
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

// 与原生快照并列的 Perfetto 事件投影；时间单位沿用录制器的微秒契约。
#[derive(Serialize)]
struct PerfettoTrace<'a> {
    #[serde(rename = "traceEvents")]
    trace_events: Vec<PerfettoEvent<'a>>,
}

#[derive(Serialize)]
struct PerfettoEvent<'a> {
    name: &'a str,
    cat: &'a str,
    ph: &'static str,
    ts: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    dur: Option<u64>,
    pid: u32,
    tid: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<PerfettoArgs<'a>>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum PerfettoArgs<'a> {
    Frame {
        frame_index: u64,
        over_budget: bool,
    },
    Span {
        path: &'a str,
        frame_index: Option<u64>,
        depth: u16,
    },
    Counter {
        value: f64,
    },
}

fn perfetto_trace(snapshot: &ProfileSnapshot) -> PerfettoTrace<'_> {
    let event_capacity = snapshot
        .frames
        .len()
        .saturating_add(snapshot.spans.len())
        .saturating_add(snapshot.counters.len());
    let mut events = Vec::with_capacity(event_capacity);
    for frame in &snapshot.frames {
        events.push(PerfettoEvent {
            name: &frame.name,
            cat: "frame",
            ph: "X",
            ts: frame.start_us,
            dur: Some(frame.duration_us),
            pid: 1,
            tid: &frame.stream,
            id: None,
            args: Some(PerfettoArgs::Frame {
                frame_index: frame.frame_index,
                over_budget: frame.over_budget,
            }),
        });
    }
    for span in &snapshot.spans {
        events.push(PerfettoEvent {
            name: &span.name,
            cat: &span.category,
            ph: "X",
            ts: span.start_us,
            dur: Some(span.duration_us),
            pid: 1,
            tid: &span.stream,
            id: None,
            args: Some(PerfettoArgs::Span {
                path: &span.path,
                frame_index: span.frame_index,
                depth: span.depth,
            }),
        });
    }
    for counter in &snapshot.counters {
        events.push(PerfettoEvent {
            name: &counter.name,
            cat: "counter",
            ph: "C",
            ts: counter.timestamp_us,
            dur: None,
            pid: 1,
            tid: &counter.stream,
            id: None,
            args: Some(PerfettoArgs::Counter {
                value: counter.value,
            }),
        });
    }
    PerfettoTrace {
        trace_events: events,
    }
}

fn summary_markdown(
    snapshot: &ProfileSnapshot,
    hotspots: &HotspotReport,
    counter_hotspots: &CounterHotspotReport,
    ui_hotspots: &UiHotspotReport,
) -> String {
    let over_budget = snapshot
        .frames
        .iter()
        .filter(|frame| frame.over_budget)
        .count();
    let mut summary = format!(
        "# Zircon Profile Summary\n\n- Session: `{}`\n- Frames: {}\n- Spans: {}\n- Counters: {}\n- Frame budget: {:.2} ms\n- Over-budget frames: {}\n",
        snapshot.session_id,
        snapshot.frames.len(),
        snapshot.spans.len(),
        snapshot.counters.len(),
        snapshot.frame_budget_ms,
        over_budget
    );
    let first_fixes = first_fix_candidates(hotspots, counter_hotspots, ui_hotspots);
    if !first_fixes.is_empty() {
        summary.push_str("\n## First Fix Candidates\n");
        for candidate in first_fixes {
            summary.push_str(&format!("- {candidate}\n"));
        }
    }
    summary.push_str("\n## Top Hotspots\n");
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
    if !counter_hotspots.counters.is_empty() {
        summary.push_str("\n## Counter Hotspots\n");
        for entry in counter_hotspots.counters.iter().take(10) {
            summary.push_str(&format!(
                "- `{}` total {:.2}, avg {:.2}, p95 {:.2}, max {:.2}, latest {:.2}, count {}, frames {}\n",
                entry.path,
                entry.total,
                entry.avg,
                entry.p95,
                entry.max,
                entry.latest,
                entry.count,
                entry.frame_count
            ));
        }
    }
    if !ui_hotspots.scenarios.is_empty() {
        summary.push_str("\n## UI Hotspots\n");
        for scenario in &ui_hotspots.scenarios {
            let damage_coverage_percent = if scenario.presented_surface_pixels > 0 {
                scenario.painted_pixels as f64 * 100.0 / scenario.presented_surface_pixels as f64
            } else {
                0.0
            };
            summary.push_str(&format!(
                "- `{}` frames={} p95={:.2} ms max={:.2} ms host_transactions={} host_scopes={} legacy_dirty_transactions={} host_targets={{full:{},shell:{},workbench:{},view:{},window:{},paint:{}}} slow_path={} presentation={} render={} chrome_snapshot={} model_build={} command_full={} command_patch={} softbuffer_present={} gpu_upload_bytes={} gpu_draw_calls={} gpu_timestamp_supported_presents={} gpu_time_samples={} gpu_time_p50={:.3} ms gpu_time_p95={:.3} ms gpu_time_max={:.3} ms gpu_profile_latency_max_frames={} gpu_visible_commands={} gpu_visible_draw_items={} gpu_batch_layers={} gpu_batch_dependencies={} redraw_full={} redraw_region={} full_paint={} region_paint={} painted_pixels={} presented_surface_pixels={} damage_coverage={:.2}%\n",
                scenario.scenario,
                scenario.frame_count,
                scenario.frame_p95_us as f64 / 1_000.0,
                scenario.frame_max_us as f64 / 1_000.0,
                scenario.host_invalidation_transaction_count,
                scenario.host_invalidation_scope_count,
                scenario.host_invalidation_legacy_dirty_transaction_count,
                scenario.host_invalidation_full_target_count,
                scenario.host_invalidation_shell_content_target_count,
                scenario.host_invalidation_workbench_projection_target_count,
                scenario.host_invalidation_view_presentation_target_count,
                scenario.host_invalidation_window_metrics_target_count,
                scenario.host_invalidation_paint_only_target_count,
                scenario.slow_path_rebuild_count,
                scenario.presentation_rebuild_count,
                scenario.render_path_count,
                scenario.chrome_snapshot_count,
                scenario.workbench_model_build_count,
                scenario.chrome_command_full_rebuild_count,
                scenario.chrome_command_patch_count,
                scenario.software_fallback_present_count,
                scenario.gpu_upload_bytes,
                scenario.gpu_draw_calls,
                scenario.gpu_timestamp_supported_present_count,
                scenario.gpu_time_sample_count,
                scenario.gpu_time_p50_us as f64 / 1_000.0,
                scenario.gpu_time_p95_us as f64 / 1_000.0,
                scenario.gpu_time_max_us as f64 / 1_000.0,
                scenario.gpu_profile_latency_max_frames,
                scenario.gpu_visible_commands,
                scenario.gpu_visible_draw_items,
                scenario.gpu_batch_layers,
                scenario.gpu_batch_dependencies,
                scenario.redraw_full_frame_count,
                scenario.redraw_region_count,
                scenario.full_paint_count,
                scenario.region_paint_count,
                scenario.painted_pixels,
                scenario.presented_surface_pixels,
                damage_coverage_percent
            ));
        }
    }
    if !ui_hotspots.alerts.is_empty() {
        summary.push_str("\n## UI Alerts\n");
        for alert in &ui_hotspots.alerts {
            summary.push_str(&format!(
                "- `{}` `{}`: {}\n",
                alert.scenario, alert.rule, alert.message
            ));
        }
    }
    summary
}

fn first_fix_candidates(
    hotspots: &HotspotReport,
    counter_hotspots: &CounterHotspotReport,
    ui_hotspots: &UiHotspotReport,
) -> Vec<String> {
    let mut candidates = ui_hotspots
        .alerts
        .iter()
        .take(5)
        .map(|alert| {
            format!(
                "UI `{}` `{}`: {}",
                alert.scenario, alert.rule, alert.message
            )
        })
        .collect::<Vec<_>>();
    if candidates.len() >= 5 {
        return candidates;
    }

    for entry in hotspots.hotspots.iter().take(5 - candidates.len()) {
        candidates.push(format!(
            "CPU `{}` p95 {:.2} ms, total {:.2} ms over {} samples",
            entry.path,
            entry.p95_us as f64 / 1_000.0,
            entry.total_us as f64 / 1_000.0,
            entry.count
        ));
    }
    if candidates.len() >= 5 {
        return candidates;
    }

    for entry in counter_hotspots.counters.iter().take(5 - candidates.len()) {
        candidates.push(format!(
            "Counter `{}` total {:.2}, p95 {:.2} over {} samples",
            entry.path, entry.total, entry.p95, entry.count
        ));
    }
    candidates
}

#[cfg(test)]
#[path = "tests/export.rs"]
mod tests;
