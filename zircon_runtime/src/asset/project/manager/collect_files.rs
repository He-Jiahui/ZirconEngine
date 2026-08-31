use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::safe_project_path::is_link_or_reparse;
use crate::asset::AssetImportError;

use super::is_meta_sidecar::is_meta_sidecar;

pub(super) fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), AssetImportError> {
    collect_matching_files(root, files, |path| {
        !is_meta_sidecar(path)
            && !crate::core::resource::io::is_atomic_write_transaction_path(path)
            && !is_auxiliary_source_file(path)
    })
}

pub(super) fn collect_matching_files<F>(
    root: &Path,
    files: &mut Vec<PathBuf>,
    mut include: F,
) -> Result<(), AssetImportError>
where
    F: FnMut(&Path) -> bool,
{
    if !root.exists() {
        return Ok(());
    }
    collect_matching_files_recursive(root, files, &mut include)
}

fn collect_matching_files_recursive<F>(
    directory: &Path,
    files: &mut Vec<PathBuf>,
    include: &mut F,
) -> Result<(), AssetImportError>
where
    F: FnMut(&Path) -> bool,
{
    let metadata = fs::symlink_metadata(directory)?;
    reject_link_or_reparse(directory, &metadata)?;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        reject_link_or_reparse(&path, &metadata)?;
        if metadata.is_dir() {
            collect_matching_files_recursive(&path, files, include)?;
        } else if metadata.is_file() && include(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn reject_link_or_reparse(path: &Path, metadata: &fs::Metadata) -> Result<(), AssetImportError> {
    if is_link_or_reparse(metadata) {
        return Err(AssetImportError::UnsafeProjectAssetLink {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

fn is_auxiliary_source_file(path: &Path) -> bool {
    // External glTF buffers and raw font binaries are source auxiliaries, not standalone assets.
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(is_auxiliary_source_extension)
}

fn is_auxiliary_source_extension(extension: &str) -> bool {
    match extension.len() {
        3 => ["bin", "ttf", "otf"]
            .iter()
            .any(|candidate| extension.eq_ignore_ascii_case(candidate)),
        4 => extension.eq_ignore_ascii_case("woff"),
        5 => extension.eq_ignore_ascii_case("woff2"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::hint::black_box;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Instant;

    use super::{collect_files, is_auxiliary_source_extension, is_auxiliary_source_file};

    static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn source_collection_ignores_atomic_write_transaction_siblings() {
        let root = std::env::temp_dir().join(format!(
            "zircon_collect_files_atomic_siblings_{}_{}",
            std::process::id(),
            NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("material.zmaterial");
        let staging = root.join(".material.zmaterial.zr-staging-123-4");
        let backup = root.join(".material.zmaterial.zr-backup-123-5");
        fs::write(&source, "source").unwrap();
        fs::write(&staging, "staging").unwrap();
        fs::write(&backup, "backup").unwrap();

        let mut files = Vec::new();
        collect_files(&root, &mut files).unwrap();

        assert_eq!(files, vec![source]);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn optimization_batch_gb_runtime484_auxiliary_extension_dispatch_preserves_supported_set() {
        for extension in ["bin", "BIN", "ttf", "OTF", "woff", "WoFf2"] {
            assert!(is_auxiliary_source_extension(extension), "{extension}");
        }
        for extension in ["", "png", "woff22", "font"] {
            assert!(!is_auxiliary_source_extension(extension), "{extension}");
        }
        assert!(is_auxiliary_source_file(Path::new("fonts/interface.WOFF2")));
        assert!(!is_auxiliary_source_file(Path::new(
            "textures/interface.png"
        )));
    }

    const CHECKS_PER_SAMPLE: usize = 1_048_576;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_gb_runtime484_auxiliary_extension_dispatch_benchmark() {
        const INPUT: &str = "woff2";
        for _ in 0..4 {
            black_box(measure_checks(INPUT, false));
            black_box(measure_checks(INPUT, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_checks(INPUT, false));
                optimized_samples.push(measure_checks(INPUT, true));
            } else {
                optimized_samples.push(measure_checks(INPUT, true));
                legacy_samples.push(measure_checks(INPUT, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME484_AUXILIARY_EXTENSION_DISPATCH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} value_bytes={} checks_per_sample={CHECKS_PER_SAMPLE} legacy_candidate_comparisons_per_check=5 optimized_candidate_comparisons_per_check=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            INPUT.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 75 / 100);
    }

    fn measure_checks(input: &str, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            let matched = if optimized {
                is_auxiliary_source_extension(black_box(input))
            } else {
                legacy_is_auxiliary_source_extension(black_box(input))
            };
            black_box(matched);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_is_auxiliary_source_extension(extension: &str) -> bool {
        extension.eq_ignore_ascii_case("bin")
            || extension.eq_ignore_ascii_case("ttf")
            || extension.eq_ignore_ascii_case("otf")
            || extension.eq_ignore_ascii_case("woff")
            || extension.eq_ignore_ascii_case("woff2")
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
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
