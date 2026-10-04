use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;
use zircon_runtime_interface::{
    ProfileFrameSnapshot, ProfileSnapshot, ProfileSpanSnapshot, PROFILE_COUNTER_HOTSPOTS_FILE,
    PROFILE_HOTSPOTS_FILE, PROFILE_TIMELINE_NATIVE_FILE, PROFILE_TIMELINE_PERFETTO_FILE,
    PROFILE_UI_HOTSPOTS_FILE,
};

use super::{export_snapshot, perfetto_trace, write_json, ProfileExportError};

fn managed_test_root(label: &str) -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .expect("managed profile tests require CARGO_TARGET_DIR")
        .join(format!(
            "zircon-profile-json-{label}-{}",
            std::process::id()
        ))
}

fn sample_snapshot(span_count: usize, output_root: &Path) -> ProfileSnapshot {
    let mut snapshot = ProfileSnapshot {
        session_id: format!("streaming-json-{span_count}"),
        output_root: output_root.to_string_lossy().into_owned(),
        ..ProfileSnapshot::default()
    };
    snapshot.frames.push(ProfileFrameSnapshot {
        stream: "runtime".to_string(),
        name: "frame\n\"escaped\"".to_string(),
        frame_index: 0,
        start_us: 1,
        duration_us: 2,
        budget_ms: 16.67,
        over_budget: false,
    });
    snapshot.spans.reserve(span_count);
    for index in 0..span_count {
        snapshot.spans.push(ProfileSpanSnapshot {
            id: index as u64,
            parent_id: None,
            frame_index: Some(0),
            stream: "runtime".to_string(),
            category: "render".to_string(),
            name: "submit\n\"escaped\"".to_string(),
            path: "runtime/render:submit".to_string(),
            start_us: index as u64,
            duration_us: 3,
            depth: 1,
        });
    }
    snapshot
}

#[test]
fn profile_json_export_preserves_pretty_json_bytes_with_and_without_perfetto() {
    for include_perfetto in [false, true] {
        let root = managed_test_root(&format!("equivalence-{include_perfetto}"));
        let _ = fs::remove_dir_all(&root);
        let snapshot = sample_snapshot(3, &root);
        let report = export_snapshot(&snapshot, include_perfetto).expect("profile export");
        let exported = Path::new(&report.export_dir);

        let native = fs::read(exported.join(PROFILE_TIMELINE_NATIVE_FILE)).expect("native JSON");
        assert_eq!(native, serde_json::to_vec_pretty(&snapshot).unwrap());

        let hotspot_bytes = fs::read(exported.join(PROFILE_HOTSPOTS_FILE)).expect("hotspot JSON");
        assert_eq!(
            hotspot_bytes,
            serde_json::to_vec_pretty(&report.hotspots).unwrap()
        );
        let counter_bytes =
            fs::read(exported.join(PROFILE_COUNTER_HOTSPOTS_FILE)).expect("counter hotspot JSON");
        assert_eq!(
            counter_bytes,
            serde_json::to_vec_pretty(&report.counter_hotspots).unwrap()
        );
        let ui_bytes = fs::read(exported.join(PROFILE_UI_HOTSPOTS_FILE)).expect("UI hotspot JSON");
        assert_eq!(
            ui_bytes,
            serde_json::to_vec_pretty(&report.ui_hotspots).unwrap()
        );

        if include_perfetto {
            let trace = perfetto_trace(&snapshot);
            let perfetto =
                fs::read(exported.join(PROFILE_TIMELINE_PERFETTO_FILE)).expect("Perfetto JSON");
            assert_eq!(perfetto, serde_json::to_vec_pretty(&trace).unwrap());
        } else {
            assert!(!exported.join(PROFILE_TIMELINE_PERFETTO_FILE).exists());
        }

        let _ = fs::remove_dir_all(&root);
    }
}

#[test]
fn profile_json_export_reports_typed_write_failure() {
    let root = managed_test_root("write-error");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create failure fixture root");
    fs::create_dir(root.join(PROFILE_TIMELINE_NATIVE_FILE))
        .expect("block JSON write with directory");
    let snapshot = sample_snapshot(1, &root);

    let error = write_json(&root, PROFILE_TIMELINE_NATIVE_FILE, &snapshot).unwrap_err();
    assert!(matches!(&error, ProfileExportError::WriteFile { .. }));
    assert!(error.to_string().contains(PROFILE_TIMELINE_NATIVE_FILE));

    let _ = fs::remove_dir_all(root);
}

#[derive(Debug)]
struct SerializationFailure;

impl Serialize for SerializationFailure {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom(
            "intentional serialization failure",
        ))
    }
}

#[test]
fn profile_json_export_preserves_typed_serialization_failure() {
    let root = managed_test_root("serialize-error");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create serialization fixture root");

    let error = write_json(&root, PROFILE_HOTSPOTS_FILE, &SerializationFailure).unwrap_err();
    assert!(matches!(&error, ProfileExportError::JsonSerialize { .. }));

    let _ = fs::remove_dir_all(root);
}

struct FailAfterBytes {
    allowed: usize,
    accepted: usize,
}

impl Write for FailAfterBytes {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.accepted >= self.allowed {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "forced write failure",
            ));
        }
        let written = bytes.len().min(self.allowed - self.accepted);
        self.accepted += written;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn profile_json_streaming_writer_reports_a_mid_stream_io_failure() {
    let snapshot = sample_snapshot(8, Path::new("."));
    let mut writer = FailAfterBytes {
        allowed: 128,
        accepted: 0,
    };
    let error = super::json_writer::write_pretty_json(
        &mut writer,
        Path::new("forced-write-error.json"),
        "forced-write-error.json",
        &snapshot,
    )
    .unwrap_err();
    match error {
        ProfileExportError::WriteFile { source, .. } => {
            assert_eq!(source.kind(), io::ErrorKind::BrokenPipe);
        }
        other => panic!("expected typed write error, got {other:?}"),
    }
}

struct FailOnFlush;

impl Write for FailOnFlush {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "forced underlying flush failure",
        ))
    }
}

#[test]
fn profile_json_streaming_writer_reports_a_flush_failure() {
    let snapshot = sample_snapshot(1, Path::new("."));
    let error = super::json_writer::write_buffered_json(
        FailOnFlush,
        Path::new("forced-flush-error.json"),
        "forced-flush-error.json",
        &snapshot,
    )
    .unwrap_err();
    match error {
        ProfileExportError::WriteFile { source, .. } => {
            assert_eq!(source.kind(), io::ErrorKind::BrokenPipe);
        }
        other => panic!("expected typed flush error, got {other:?}"),
    }
}

#[test]
#[ignore = "Release performance evidence; run through the validation coordinator"]
fn profile_json_streaming_export_release_samples() {
    const WARMUPS: usize = 5;
    const PAIRS: usize = 31;
    for span_count in [1_024, 32_768, 131_072] {
        let root = managed_test_root(&format!("release-{span_count}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create benchmark root");
        let snapshot = sample_snapshot(span_count, &root);
        for include_perfetto in [false, true] {
            let trace = include_perfetto.then(|| perfetto_trace(&snapshot));
            let mut legacy_samples = Vec::with_capacity(PAIRS);
            let mut streaming_samples = Vec::with_capacity(PAIRS);
            for iteration in 0..WARMUPS + PAIRS {
                let legacy_path = root.join("legacy.json");
                let streaming_path = root.join("streaming.json");
                let timed = iteration >= WARMUPS;
                let mut run_legacy = || {
                    let started = Instant::now();
                    let bytes = if let Some(trace) = &trace {
                        serde_json::to_vec_pretty(trace).expect("legacy Perfetto JSON")
                    } else {
                        serde_json::to_vec_pretty(&snapshot).expect("legacy native JSON")
                    };
                    fs::write(&legacy_path, bytes).expect("legacy write");
                    started.elapsed().as_nanos()
                };
                let mut run_streaming = || {
                    let started = Instant::now();
                    if let Some(trace) = &trace {
                        write_json(&root, "streaming.json", trace).expect("streamed Perfetto JSON");
                    } else {
                        write_json(&root, "streaming.json", &snapshot)
                            .expect("streamed native JSON");
                    }
                    started.elapsed().as_nanos()
                };
                let (legacy_ns, streaming_ns) = if iteration % 2 == 0 {
                    (run_legacy(), run_streaming())
                } else {
                    let streaming_ns = run_streaming();
                    let legacy_ns = run_legacy();
                    (legacy_ns, streaming_ns)
                };
                if timed {
                    let legacy = fs::read(&legacy_path).expect("legacy bytes");
                    let streaming = fs::read(&streaming_path).expect("streamed bytes");
                    assert_eq!(legacy, streaming);
                    legacy_samples.push(legacy_ns);
                    streaming_samples.push(streaming_ns);
                    println!(
                        "RUNTIME03_PROFILE_JSON_STREAMING_BENCH_V1 spans={span_count} perfetto={include_perfetto} pair={} legacy_ns={legacy_ns} streaming_ns={streaming_ns} bytes={}",
                        iteration - WARMUPS,
                        legacy.len()
                    );
                }
            }
            legacy_samples.sort_unstable();
            streaming_samples.sort_unstable();
            println!(
                "RUNTIME03_PROFILE_JSON_STREAMING_BENCH_SUMMARY_V1 spans={span_count} perfetto={include_perfetto} pairs={PAIRS} legacy_p50_ns={} legacy_p95_ns={} legacy_p99_ns={} streaming_p50_ns={} streaming_p95_ns={} streaming_p99_ns={}",
                legacy_samples[15],
                legacy_samples[29],
                legacy_samples[30],
                streaming_samples[15],
                streaming_samples[29],
                streaming_samples[30],
            );
        }
        let _ = fs::remove_dir_all(root);
    }
}
