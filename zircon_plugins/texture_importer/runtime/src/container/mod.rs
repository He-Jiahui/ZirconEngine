use zircon_runtime::asset::{AssetImportContext, AssetImportError};
use zircon_runtime::core::framework::render::RenderImageDimension;

mod astc;
mod dds;
mod ktx;
mod support;

#[cfg(test)]
use support::*;

pub(crate) struct TextureContainerInfo {
    pub(crate) format: String,
    /// Rewritten container bytes when the importer expands standard KTX2 supercompression.
    pub(crate) upload_bytes: Option<Vec<u8>>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) dimension: RenderImageDimension,
    pub(crate) depth_or_array_layers: u32,
    pub(crate) mip_count: u32,
    pub(crate) array_layers: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TextureContainerKind {
    Dds,
    Ktx1,
    Ktx2,
    Astc,
}

impl TextureContainerKind {
    fn from_extension(extension: &str) -> Option<Self> {
        if extension.eq_ignore_ascii_case("dds") {
            Some(Self::Dds)
        } else if extension.eq_ignore_ascii_case("ktx") {
            Some(Self::Ktx1)
        } else if extension.eq_ignore_ascii_case("ktx2") {
            Some(Self::Ktx2)
        } else if extension.eq_ignore_ascii_case("astc") {
            Some(Self::Astc)
        } else {
            None
        }
    }
}

pub(crate) fn parse_container_info(
    context: &AssetImportContext,
) -> Result<TextureContainerInfo, AssetImportError> {
    let extension = context
        .source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();
    match TextureContainerKind::from_extension(extension) {
        Some(TextureContainerKind::Dds) => dds::parse(context),
        Some(TextureContainerKind::Ktx1) => ktx::parse_ktx1(context),
        Some(TextureContainerKind::Ktx2) => ktx::parse_ktx2(context),
        Some(TextureContainerKind::Astc) => astc::parse(context),
        None => Err(AssetImportError::UnsupportedFormat(format!(
            "texture container importer does not handle {}",
            context.source_path.display()
        ))),
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod plugins07_container_hotpath_tests {
    use std::{hint::black_box, time::Instant};

    use super::*;

    const EXTENSIONS: [&str; 8] = ["DDS", "dds", "KtX", "ktx", "KTX2", "ktx2", "AstC", "astc"];

    fn legacy_container_kind(extension: &str) -> Option<TextureContainerKind> {
        match extension.to_ascii_lowercase().as_str() {
            "dds" => Some(TextureContainerKind::Dds),
            "ktx" => Some(TextureContainerKind::Ktx1),
            "ktx2" => Some(TextureContainerKind::Ktx2),
            "astc" => Some(TextureContainerKind::Astc),
            _ => None,
        }
    }

    #[test]
    fn plugins07_container_hotpath_dispatch_preserves_ascii_case_insensitive_matching() {
        for extension in EXTENSIONS {
            assert_eq!(
                TextureContainerKind::from_extension(extension),
                legacy_container_kind(extension),
            );
        }
        assert_eq!(TextureContainerKind::from_extension("png"), None);
    }

    #[test]
    #[ignore = "release-only container extension dispatch benchmark"]
    fn plugins07_container_hotpath_release_borrowed_extension_dispatch_p95_gate() {
        const SAMPLE_PAIRS: usize = 21;
        const CHECKS_PER_SAMPLE: usize = 100_000;

        fn measure_legacy() -> u128 {
            let started = Instant::now();
            for check in 0..CHECKS_PER_SAMPLE {
                let extension = black_box(EXTENSIONS[check % EXTENSIONS.len()]);
                black_box(legacy_container_kind(extension));
            }
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized() -> u128 {
            let started = Instant::now();
            for check in 0..CHECKS_PER_SAMPLE {
                let extension = black_box(EXTENSIONS[check % EXTENSIONS.len()]);
                black_box(TextureContainerKind::from_extension(extension));
            }
            started.elapsed().as_nanos().max(1)
        }

        for _ in 0..4 {
            black_box(measure_legacy());
            black_box(measure_optimized());
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy());
                optimized_samples.push(measure_optimized());
            } else {
                optimized_samples.push(measure_optimized());
                legacy_samples.push(measure_legacy());
            }
        }

        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        let improvement_percent = improvement_percent(legacy_p95_ns, optimized_p95_ns);
        println!(
            "PERF_RESULT plugins07_container_extension_dispatch sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} order=alternating_legacy_first_even \
legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_owned_extension_strings_per_sample={CHECKS_PER_SAMPLE} \
optimized_owned_extension_strings_per_sample=0 legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} \
threshold_percent=50 legacy_ns={} optimized_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
            "borrowed extension dispatch must reduce P95 by at least 50%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn improvement_percent(legacy: u128, optimized: u128) -> u128 {
        if optimized >= legacy {
            0
        } else {
            legacy.saturating_sub(optimized).saturating_mul(100) / legacy.max(1)
        }
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
