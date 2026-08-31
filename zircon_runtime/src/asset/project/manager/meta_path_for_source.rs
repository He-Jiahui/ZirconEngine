use std::path::{Path, PathBuf};

pub(super) fn meta_path_for_source(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("asset");
    let mut meta_file_name = String::with_capacity(file_name.len() + ".zmeta".len());
    meta_file_name.push_str(file_name);
    meta_file_name.push_str(".zmeta");
    path.with_file_name(meta_file_name)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const PATHS_PER_SAMPLE: usize = 262_144;

    #[test]
    fn optimization_batch_ex_runtime456_preserves_meta_sidecar_paths() {
        for path in [
            Path::new("assets/mesh.zmesh"),
            Path::new("asset"),
            Path::new("assets/ui/main menu.zui"),
            Path::new(""),
        ] {
            assert_eq!(meta_path_for_source(path), legacy_meta_path(path));
        }

        let production = include_str!("meta_path_for_source.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        assert!(!production.contains("format!("));
        assert!(production.contains("String::with_capacity"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_ex_runtime456_direct_meta_file_name_benchmark() {
        let path = Path::new(
            "assets/generated/materials/character_surface_variant_with_a_long_stable_source_name.zmaterial",
        );
        for _ in 0..4 {
            black_box(measure_paths(path, legacy_meta_path));
            black_box(measure_paths(path, meta_path_for_source));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_paths(path, legacy_meta_path));
                optimized_samples.push(measure_paths(path, meta_path_for_source));
            } else {
                optimized_samples.push(measure_paths(path, meta_path_for_source));
                legacy_samples.push(measure_paths(path, legacy_meta_path));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn legacy_meta_path(path: &Path) -> PathBuf {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("asset");
        path.with_file_name(format!("{file_name}.zmeta"))
    }

    fn measure_paths(path: &Path, mut build: impl FnMut(&Path) -> PathBuf) -> u128 {
        let started = Instant::now();
        let mut total_len = 0_usize;
        for _ in 0..PATHS_PER_SAMPLE {
            let meta_path = build(black_box(path));
            total_len += black_box(meta_path.as_os_str().len());
            black_box(meta_path);
        }
        black_box(total_len);
        started.elapsed().as_nanos().max(1)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME456_DIRECT_META_FILE_NAME_BENCH_V1 sample_pairs={SAMPLE_PAIRS} paths_per_sample={PATHS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=10",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(90) / 100,
            "direct metadata file-name construction must reduce P95 by at least 10%"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
