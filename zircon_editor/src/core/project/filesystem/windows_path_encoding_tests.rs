use std::hint::black_box;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::encode_nul_terminated_path;

const MARKER: &str = "EDITOR184_SCENE_GUARD_PATH_SINGLE_ALLOCATION_ENCODING_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const REPEATS: usize = 8_192;

#[test]
fn optimization_batch_20260826gr_editor184_scene_guard_path_preserves_utf16_and_single_nul() {
    let path = Path::new("C:\\Zircon\\caf\u{e9}\\scene.zscene");
    let encoded = encode_nul_terminated_path(path);
    let expected = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    assert_eq!(encoded, expected);
    assert_eq!(encoded.last(), Some(&0));
    assert!(!encoded[..encoded.len() - 1].contains(&0));
}

#[test]
fn optimization_batch_20260826gr_editor184_scene_guard_collects_terminator_once() {
    let source = include_str!("../filesystem.rs");
    assert!(source.contains("encode_nul_terminated_path(path)"));
    assert!(source.contains(".chain(std::iter::once(0))"));
    assert!(!source.contains("wide_path.push(0)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gr_editor184_scene_guard_path_single_allocation_encoding_bench() {
    let path = benchmark_path();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&path, legacy_encode));
            optimized_samples.push(measure(&path, encode_nul_terminated_path));
        } else {
            optimized_samples.push(measure(&path, encode_nul_terminated_path));
            legacy_samples.push(measure(&path, legacy_encode));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-allocation encoding must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn benchmark_path() -> PathBuf {
    let mut path = PathBuf::from("C:\\Zircon");
    for index in 0..24 {
        path.push(format!("scene_asset_segment_{index:02}"));
    }
    path.push("main.scene.toml");
    path
}

fn legacy_encode(path: &Path) -> Vec<u16> {
    let mut encoded = path.as_os_str().encode_wide().collect::<Vec<_>>();
    encoded.push(0);
    encoded
}

fn measure(path: &Path, implementation: fn(&Path) -> Vec<u16>) -> u64 {
    let started = Instant::now();
    let mut units = 0;
    for _ in 0..REPEATS {
        units += implementation(black_box(path)).len();
    }
    black_box(units);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
