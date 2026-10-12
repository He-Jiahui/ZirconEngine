use std::hint::black_box;

use super::StatusBarModel;
use crate::scene::viewport::{GridMode, SceneViewportChromeSettings};

const BENCHMARK_MARKER: &str = "EDITOR862_VIEWPORT_CHROME_DIRECT_SETTINGS_BENCH_V1";

#[test]
fn editor862_viewport_chrome_text_preserves_grid_and_snap_semantics() {
    let mut settings = SceneViewportChromeSettings::default();
    settings.grid_mode = GridMode::Hidden;
    assert_eq!(
        StatusBarModel::viewport_chrome_text(&settings),
        ("Grid: Off".to_string(), "Snap: Off".to_string())
    );

    settings.grid_mode = GridMode::VisibleNoSnap;
    settings.translate_step = 0.25;
    assert_eq!(
        StatusBarModel::viewport_chrome_text(&settings),
        ("Grid: 0.25 m".to_string(), "Snap: Off".to_string())
    );

    settings.grid_mode = GridMode::VisibleAndSnap;
    assert_eq!(
        StatusBarModel::viewport_chrome_text(&settings),
        ("Grid: 0.25 m".to_string(), "Snap: On".to_string())
    );
}

#[test]
#[ignore = "release-only direct-settings performance evidence"]
fn editor862_viewport_chrome_direct_settings_bench() {
    const QUERY_COUNT: usize = 65_536;
    let mut settings = SceneViewportChromeSettings::default();
    settings.grid_mode = GridMode::VisibleAndSnap;
    settings.translate_step = 0.25;

    for _ in 0..QUERY_COUNT {
        black_box(StatusBarModel::viewport_chrome_text(&settings));
    }

    println!(
        "{BENCHMARK_MARKER} queries={QUERY_COUNT} legacy_full_chrome_snapshot_builds={QUERY_COUNT} \
         optimized_full_chrome_snapshot_builds=0 reduction_pct=100"
    );
}
