use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::sidecar_resource_uri;

const SAMPLE_PAIRS: usize = 101;
const URIS_PER_SAMPLE: usize = 4_096;

#[test]
fn runtime879_sidecar_uri_preserves_components_and_lossy_text() {
    for path in [
        PathBuf::new(),
        PathBuf::from("."),
        PathBuf::from("models/mesh.bin"),
        PathBuf::from("models//./nested/../mesh.bin"),
        PathBuf::from("中文/🦀.zasset"),
        PathBuf::from("a/b/c/d/e"),
    ] {
        assert_eq!(sidecar_resource_uri(&path), legacy_uri(&path), "{path:?}");
    }
}

#[cfg(windows)]
#[test]
fn runtime879_sidecar_uri_preserves_invalid_windows_unicode_lossiness() {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    let path = PathBuf::from(OsString::from_wide(&[0x0061, 0x005c, 0xd800, 0x0062]));
    assert_eq!(sidecar_resource_uri(&path), legacy_uri(&path));
}

#[test]
#[ignore = "release percentile evidence; run through the managed Windows validation lane"]
fn runtime879_sidecar_uri_direct_release_percentiles() {
    let mut path = PathBuf::new();
    for index in 0..32 {
        path.push(format!("asset_component_{index:02}"));
    }
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&path, legacy_uri));
            optimized.push(measure(&path, sidecar_resource_uri));
        } else {
            optimized.push(measure(&path, sidecar_resource_uri));
            legacy.push(measure(&path, legacy_uri));
        }
    }
    let legacy_p50_ns = percentile(&mut legacy.clone(), 50);
    let legacy_p95_ns = percentile(&mut legacy.clone(), 95);
    let legacy_p99_ns = percentile(&mut legacy, 99);
    let optimized_p50_ns = percentile(&mut optimized.clone(), 50);
    let optimized_p95_ns = percentile(&mut optimized.clone(), 95);
    let optimized_p99_ns = percentile(&mut optimized, 99);
    println!(
        "RUNTIME879_SIDECAR_URI_DIRECT_BENCH_V1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} optimized_p99_ns={optimized_p99_ns}"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns.saturating_mul(110).div_ceil(100));
}

fn legacy_uri(relative: &Path) -> String {
    let relative = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    format!("res://{relative}")
}

fn measure(path: &Path, make_uri: fn(&Path) -> String) -> Duration {
    let started = Instant::now();
    let checksum = (0..URIS_PER_SAMPLE)
        .map(|_| black_box(make_uri(black_box(path))).len())
        .sum::<usize>();
    black_box(checksum);
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percent: usize) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * percent).div_ceil(100).saturating_sub(1)].as_nanos()
}
