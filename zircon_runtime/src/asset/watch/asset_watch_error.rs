use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetWatchError {
    pub assets_root: PathBuf,
    pub paths: Vec<PathBuf>,
    pub message: String,
}

impl AssetWatchError {
    pub(super) fn from_notify_error(assets_root: PathBuf, error: notify::Error) -> Self {
        let message = error.to_string();
        Self {
            assets_root,
            paths: error.paths,
            message,
        }
    }

    pub(crate) fn from_message(assets_root: PathBuf, message: impl Into<String>) -> Self {
        Self {
            assets_root,
            paths: Vec::new(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod optimization_batch_gt_runtime575_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 31;
    const ERRORS_PER_SAMPLE: usize = 1_024;
    const PATHS_PER_ERROR: usize = 4;

    #[test]
    fn optimization_batch_gt_runtime575_watch_error_preserves_paths_and_message() {
        let assets_root = PathBuf::from("C:/project/assets");
        let first = assets_root.join("first.zasset");
        let second = assets_root.join("second.zasset");
        let notify_error = notify::Error::path_not_found()
            .add_path(first.clone())
            .add_path(second.clone());
        let expected_message = notify_error.to_string();

        let error = AssetWatchError::from_notify_error(assets_root.clone(), notify_error);

        assert_eq!(error.assets_root, assets_root);
        assert_eq!(error.paths, vec![first, second]);
        assert_eq!(error.message, expected_message);

        let production = include_str!("asset_watch_error.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(production.contains("paths: error.paths,"));
        assert!(!production.contains("paths: error.paths.clone()"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_gt_runtime575_watch_error_path_move_p95() {
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure(false));
                optimized_samples.push(measure(true));
            } else {
                optimized_samples.push(measure(true));
                legacy_samples.push(measure(false));
            }
        }

        let legacy_p95_ns = p95(&mut legacy_samples);
        let optimized_p95_ns = p95(&mut optimized_samples);
        println!(
            "RUNTIME575_WATCH_ERROR_PATH_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} errors_per_sample={ERRORS_PER_SAMPLE} paths_per_error={PATHS_PER_ERROR} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(90),
            "expected moving notify paths to lower p95 by at least 10%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn measure(optimized: bool) -> u128 {
        let errors = fixture_errors();
        let started = Instant::now();
        let checksum = errors.into_iter().fold(0_usize, |checksum, error| {
            let converted = if optimized {
                AssetWatchError::from_notify_error(PathBuf::new(), error)
            } else {
                legacy_from_notify_error(PathBuf::new(), error)
            };
            let path_bytes = converted
                .paths
                .iter()
                .map(|path| path.as_os_str().len())
                .sum::<usize>();
            checksum ^ path_bytes ^ converted.message.len()
        });
        black_box(checksum);
        started.elapsed().as_nanos()
    }

    fn fixture_errors() -> Vec<notify::Error> {
        (0..ERRORS_PER_SAMPLE)
            .map(|error_index| {
                (0..PATHS_PER_ERROR).fold(notify::Error::path_not_found(), |error, path_index| {
                    error.add_path(PathBuf::from(format!(
                        "C:/project/assets/{error_index}/asset-{path_index}.zasset"
                    )))
                })
            })
            .collect()
    }

    fn legacy_from_notify_error(assets_root: PathBuf, error: notify::Error) -> AssetWatchError {
        AssetWatchError {
            assets_root,
            paths: error.paths.clone(),
            message: error.to_string(),
        }
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[samples.len() * 95 / 100]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
