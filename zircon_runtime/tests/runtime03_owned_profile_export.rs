#![cfg(feature = "profiling")]

use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime::core::diagnostics::profiling::{
    analyze_counter_hotspots, analyze_hotspots, analyze_ui_hotspots, export_report, record_counter,
    reset_capture, snapshot, start_capture, stop_capture, ProfileCaptureConfig, ProfileFrameScope,
    ProfileScope,
};
use zircon_runtime_interface::{
    PROFILE_COUNTER_HOTSPOTS_FILE, PROFILE_HOTSPOTS_FILE, PROFILE_SUMMARY_FILE,
    PROFILE_TIMELINE_NATIVE_FILE, PROFILE_TIMELINE_PERFETTO_FILE, PROFILE_UI_HOTSPOTS_FILE,
};

#[test]
fn public_export_report_preserves_frozen_samples_after_completion_and_capture_reset() {
    for include_perfetto in [false, true] {
        let output_root = PathBuf::from(
            std::env::var_os("CARGO_TARGET_DIR")
                .expect("profile export regression requires managed CARGO_TARGET_DIR"),
        )
        .join(format!(
            "runtime03-owned-export-{}-{include_perfetto}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&output_root);
        let config = ProfileCaptureConfig {
            session_id: "capture/祖先:\"frozen\"".to_string(),
            output_root: output_root.to_string_lossy().into_owned(),
            max_frames: 2,
            max_spans: 3,
            max_counters: 2,
            include_perfetto,
            ..ProfileCaptureConfig::default()
        };
        reset_capture();
        assert!(start_capture(config.clone()).active);
        let retired = ProfileScope::enter("retired", "old", "old_epoch");
        reset_capture();
        assert!(start_capture(config).active);

        for index in 0..5 {
            let frame = ProfileFrameScope::enter("runtime", "帧/\"tick\"");
            let parent = ProfileScope::enter("runtime", "export", "祖/先:parent");
            drop(ProfileScope::enter_named(
                "runtime",
                "export",
                format!("child_{index}\n\"叶\""),
            ));
            record_counter("runtime", "completed_count", (index + 1) as f64);
            record_counter(
                "runtime",
                "ui.owned_export.frame_duration_us",
                (index + 1) as f64,
            );
            drop(parent);
            drop(frame);
        }
        std::thread::spawn(|| {
            let frame = ProfileFrameScope::enter("worker", "worker_frame");
            drop(ProfileScope::enter("worker", "export", "worker_complete"));
            record_counter("worker", "completed_count", 7.0);
            record_counter("worker", "ui.owned_export.frame_duration_us", 7.0);
            drop(frame);
        })
        .join()
        .expect("worker capture completes");
        drop(retired);
        let live_frame = ProfileFrameScope::enter("runtime", "late_frame");
        let live_parent = ProfileScope::enter("runtime", "export", "late_parent");

        let frozen = snapshot();
        assert!(frozen.active);
        assert_eq!(frozen.frames.len(), 2);
        assert_eq!(frozen.frames[0].stream, "runtime");
        assert_eq!(frozen.frames[0].frame_index, 4);
        assert_eq!(frozen.frames[1].stream, "worker");
        assert_eq!(frozen.frames[1].frame_index, 0);
        assert_eq!(
            frozen
                .spans
                .iter()
                .map(|span| span.name.as_str())
                .collect::<Vec<_>>(),
            vec!["child_4\n\"叶\"", "祖/先:parent", "worker_complete"]
        );
        assert_eq!(frozen.spans[0].parent_id, Some(frozen.spans[1].id));
        assert_eq!(
            frozen.spans[0].frame_index,
            Some(frozen.frames[0].frame_index)
        );
        assert_eq!(frozen.spans[0].depth, 1);
        assert_eq!(
            frozen.spans[0].path,
            "runtime/export:祖/先:parent/export:child_4\n\"叶\""
        );
        assert_eq!(frozen.spans[2].stream, "worker");
        assert_eq!(frozen.spans[2].parent_id, None);
        assert_eq!(
            frozen.spans[2].frame_index,
            Some(frozen.frames[1].frame_index)
        );
        assert_eq!(frozen.counters.len(), 2);
        assert!(frozen.counters.iter().all(|counter| {
            counter.stream == "worker"
                && counter.frame_index == Some(frozen.frames[1].frame_index)
                && counter.value == 7.0
        }));
        let retention = &frozen.recorder_retention[0];
        assert_eq!(
            (retention.frames.written, retention.frames.overwritten),
            (6, 4)
        );
        assert_eq!(
            (retention.spans.written, retention.spans.overwritten),
            (11, 8)
        );
        assert_eq!(
            (retention.counters.written, retention.counters.overwritten),
            (12, 10)
        );

        let report = export_report().expect("export current capture");
        assert_eq!(report.snapshot, frozen);
        assert_eq!(report.hotspots, analyze_hotspots(&frozen));
        assert_eq!(report.counter_hotspots, analyze_counter_hotspots(&frozen));
        assert_eq!(report.ui_hotspots, analyze_ui_hotspots(&frozen));
        let expected_perfetto = include_perfetto && cfg!(feature = "profiling-chrome");
        let mut expected_files = vec![PROFILE_TIMELINE_NATIVE_FILE];
        if expected_perfetto {
            expected_files.push(PROFILE_TIMELINE_PERFETTO_FILE);
        }
        expected_files.extend([
            PROFILE_HOTSPOTS_FILE,
            PROFILE_COUNTER_HOTSPOTS_FILE,
            PROFILE_UI_HOTSPOTS_FILE,
            PROFILE_SUMMARY_FILE,
        ]);
        assert_eq!(
            report.files.iter().map(String::as_str).collect::<Vec<_>>(),
            expected_files
        );

        stop_capture();
        drop(live_parent);
        drop(live_frame);
        let completed = snapshot();
        assert_ne!(completed, frozen);
        assert_eq!(completed.frames.last().unwrap().name, "late_frame");
        assert_eq!(completed.spans.last().unwrap().name, "late_parent");
        reset_capture();
        assert!(start_capture(ProfileCaptureConfig::default()).active);
        drop(ProfileScope::enter("new_capture", "export", "new_row"));
        stop_capture();
        assert_eq!(snapshot().spans.len(), 1);
        assert_eq!(snapshot().spans[0].name, "new_row");
        assert_eq!(report.snapshot, frozen);
        let exported = Path::new(&report.export_dir);
        assert_eq!(
            fs::read(exported.join(PROFILE_TIMELINE_NATIVE_FILE)).unwrap(),
            serde_json::to_vec_pretty(&frozen).unwrap()
        );
        assert_eq!(
            exported.join(PROFILE_TIMELINE_PERFETTO_FILE).exists(),
            expected_perfetto
        );
        reset_capture();
        fs::remove_dir_all(&output_root).expect("remove managed export regression artifacts");
    }
}
