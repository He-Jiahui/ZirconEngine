use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::export::ExportTargetMode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformBundleLayout {
    pub engine_root: PathBuf,
    pub launcher: PathBuf,
    pub runtime_library: PathBuf,
    pub assets_root: PathBuf,
}

impl PlatformBundleLayout {
    pub fn expected(build_output_root: impl AsRef<Path>, target_mode: ExportTargetMode) -> Self {
        let engine_root = build_output_root.as_ref().join("ZirconEngine");
        let assets_root = engine_root.join("assets");
        let runtime_library = engine_root.join(runtime_library_name());
        let launcher = match target_mode {
            ExportTargetMode::ClientRuntime => engine_root.join(executable_name("zircon_hub")),
            ExportTargetMode::ServerRuntime => engine_root.join(executable_name("zircon_runtime")),
        };
        Self {
            engine_root,
            launcher,
            runtime_library,
            assets_root,
        }
    }

    pub fn validate(
        build_output_root: impl AsRef<Path>,
        target_mode: ExportTargetMode,
    ) -> Result<Self, PlatformBundleLayoutError> {
        let expected = Self::expected(build_output_root, target_mode);
        let engine_root = expected.engine_root;
        require_directory(&engine_root)?;
        let assets_root = expected.assets_root;
        require_directory(&assets_root)?;
        let runtime_library = expected.runtime_library;
        require_file(&runtime_library)?;
        let launcher = match target_mode {
            ExportTargetMode::ClientRuntime => {
                let editor = engine_root.join(executable_name("zircon_editor"));
                require_file(&editor)?;
                let hub = expected.launcher;
                require_file(&hub)?;
                hub
            }
            ExportTargetMode::ServerRuntime => {
                let runtime = expected.launcher;
                require_file(&runtime)?;
                runtime
            }
        };
        Ok(Self {
            engine_root,
            launcher,
            runtime_library,
            assets_root,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformBundleLayoutError {
    MissingDirectory { path: PathBuf },
    MissingFile { path: PathBuf },
}

impl fmt::Display for PlatformBundleLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDirectory { path } => write!(
                formatter,
                "platform bundle requires directory {}",
                path.display()
            ),
            Self::MissingFile { path } => {
                write!(
                    formatter,
                    "platform bundle requires file {}",
                    path.display()
                )
            }
        }
    }
}

impl Error for PlatformBundleLayoutError {}

fn require_directory(path: &Path) -> Result<(), PlatformBundleLayoutError> {
    if path.is_dir() {
        Ok(())
    } else {
        Err(PlatformBundleLayoutError::MissingDirectory {
            path: path.to_path_buf(),
        })
    }
}

fn require_file(path: &Path) -> Result<(), PlatformBundleLayoutError> {
    if path.is_file() {
        Ok(())
    } else {
        Err(PlatformBundleLayoutError::MissingFile {
            path: path.to_path_buf(),
        })
    }
}

#[cfg(target_os = "windows")]
const fn runtime_library_name() -> &'static str {
    "zircon_runtime.dll"
}

#[cfg(target_os = "macos")]
const fn runtime_library_name() -> &'static str {
    "libzircon_runtime.dylib"
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
const fn runtime_library_name() -> &'static str {
    "libzircon_runtime.so"
}

fn executable_name(stem: &str) -> String {
    format!("{stem}{}", std::env::consts::EXE_SUFFIX)
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::path::{Path, PathBuf};
    use std::time::Instant;

    use super::{ExportTargetMode, PlatformBundleLayout};

    #[test]
    fn optimization_batch_el_platform_validation_moves_expected_assets_root() {
        let source = include_str!("platform_bundle.rs");
        let implementation = source
            .split("pub fn validate(")
            .nth(1)
            .expect("platform bundle validation implementation")
            .split("#[derive(Clone, Debug, PartialEq, Eq)]")
            .next()
            .expect("bounded platform bundle validation implementation");

        assert!(implementation.contains("let assets_root = expected.assets_root"));
        assert!(!implementation.contains("let assets_root = engine_root.join(\"assets\")"));
    }

    fn layouts(root: &Path, count: usize) -> Vec<PlatformBundleLayout> {
        let layout = PlatformBundleLayout::expected(root, ExportTargetMode::ClientRuntime);
        (0..count).map(|_| layout.clone()).collect()
    }

    #[test]
    #[ignore = "release-only direct platform assets path move benchmark"]
    fn optimization_batch_el_direct_platform_assets_path_move_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const LAYOUTS_PER_SAMPLE: usize = 2_048;

        fn measure_legacy(layouts: Vec<PlatformBundleLayout>) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for layout in black_box(layouts) {
                let assets_root = black_box(layout.engine_root.join("assets"));
                checksum = checksum.wrapping_add(assets_root.as_os_str().len());
                black_box(assets_root);
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(layouts: Vec<PlatformBundleLayout>) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for layout in black_box(layouts) {
                let assets_root = black_box(layout.assets_root);
                checksum = checksum.wrapping_add(assets_root.as_os_str().len());
                black_box(assets_root);
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let root = PathBuf::from(format!(
            "C:\\{}build",
            "export-layout-segment\\".repeat(128)
        ));
        for _ in 0..4 {
            black_box(measure_legacy(layouts(&root, LAYOUTS_PER_SAMPLE)));
            black_box(measure_optimized(layouts(&root, LAYOUTS_PER_SAMPLE)));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(layouts(&root, LAYOUTS_PER_SAMPLE)));
                optimized_samples.push(measure_optimized(layouts(&root, LAYOUTS_PER_SAMPLE)));
            } else {
                optimized_samples.push(measure_optimized(layouts(&root, LAYOUTS_PER_SAMPLE)));
                legacy_samples.push(measure_legacy(layouts(&root, LAYOUTS_PER_SAMPLE)));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "EDITOR374_DIRECT_PLATFORM_ASSETS_PATH_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             layouts_per_sample={LAYOUTS_PER_SAMPLE} root_bytes={} \
             pair_order=alternating_legacy_even legacy_extra_path_allocations_per_layout=1 \
             optimized_extra_path_allocations_per_layout=0 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            root.as_os_str().len(),
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
            "direct platform assets path move must reduce P95 by at least 50%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
