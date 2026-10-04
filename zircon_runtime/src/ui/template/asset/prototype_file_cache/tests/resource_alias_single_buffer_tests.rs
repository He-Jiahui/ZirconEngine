use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::resource_alias_for_path;

const PERFORMANCE_MARKER: &str = "RUNTIME872_PROTOTYPE_RESOURCE_ALIAS_SINGLE_BUFFER_BENCH_V1";
const SAMPLE_PAIRS: usize = 101;
const ALIASES_PER_SAMPLE: usize = 4_096;

#[test]
fn runtime872_prototype_resource_alias_single_buffer_preserves_exact_text() {
    for path in [
        Path::new("C:/project/assets/ui/editor/panel.zui"),
        Path::new("project/assets/ui/\u{6e32}\u{67d3}/panel.zui"),
        Path::new("C:/project/assets/packages/demo/assets/widget.zui"),
        Path::new("C:/project/assets"),
        Path::new("C:/project/ui/editor/panel.zui"),
    ] {
        assert_eq!(
            resource_alias_for_path(path),
            legacy_resource_alias_for_path(path),
            "path={}",
            path.display()
        );
    }
    assert_eq!(
        resource_alias_for_path(Path::new("C:/project/assets/ui/editor/panel.zui")),
        Some("res://ui/editor/panel.zui".to_string())
    );
    assert_eq!(
        resource_alias_for_path(Path::new("C:/project/assets")),
        None
    );
}

#[test]
#[ignore = "release-only prototype resource alias performance gate"]
fn runtime872_prototype_resource_alias_single_buffer_release_performance() {
    let mut path = PathBuf::from("C:/project/assets");
    for index in 0..63 {
        path.push(format!("component_{index:04}"));
    }
    path.push("surface.zui");
    assert_eq!(
        resource_alias_for_path(&path),
        legacy_resource_alias_for_path(&path)
    );

    for _ in 0..8 {
        black_box(render_batch(&path, legacy_resource_alias_for_path));
        black_box(render_batch(&path, resource_alias_for_path));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| {
                render_batch(&path, legacy_resource_alias_for_path)
            }));
            optimized_samples.push(measure(|| render_batch(&path, resource_alias_for_path)));
        } else {
            optimized_samples.push(measure(|| render_batch(&path, resource_alias_for_path)));
            legacy_samples.push(measure(|| {
                render_batch(&path, legacy_resource_alias_for_path)
            }));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let legacy_p99_ns = percentile_ns(&mut legacy_samples, 99);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    let optimized_p99_ns = percentile_ns(&mut optimized_samples, 99);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p99_ns={optimized_p99_ns} sample_pairs={SAMPLE_PAIRS} aliases_per_sample={ALIASES_PER_SAMPLE} components_per_alias=64 legacy_reference_slots_per_sample=262144 optimized_reference_slots_per_sample=0 legacy_join_outputs_per_sample=4096 optimized_join_outputs_per_sample=0"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(110),
        "single-buffer P95 {optimized_p95_ns}ns must be at most 110% of collect/join P95 {legacy_p95_ns}ns"
    );
}

fn legacy_resource_alias_for_path(path: &Path) -> Option<String> {
    let asset_root = legacy_asset_root_for_path(path)?;
    let relative = path.strip_prefix(asset_root).ok()?;
    let parts = relative
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| format!("res://{}", parts.join("/")))
}

fn legacy_asset_root_for_path(path: &Path) -> Option<&Path> {
    path.ancestors()
        .find(|ancestor| ancestor.file_name().and_then(|name| name.to_str()) == Some("assets"))
}

fn render_batch(path: &Path, render: fn(&Path) -> Option<String>) -> usize {
    (0..ALIASES_PER_SAMPLE)
        .map(|_| {
            black_box(render(black_box(path)))
                .as_deref()
                .map_or(0, str::len)
        })
        .sum()
}

fn measure<T>(run: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(run());
    started.elapsed()
}

fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)].as_nanos()
}
