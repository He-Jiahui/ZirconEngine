use crate::asset::assets::{ImportedAsset, UiThemeAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_ui_theme_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_str()?;
    let asset = UiThemeAsset::from_toml_str(document).map_err(|source| {
        AssetImportError::UiThemeDocument {
            context: "parse ui theme asset",
            source,
        }
    })?;

    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::UiTheme(asset),
    ))
}

#[cfg(test)]
mod plugins07_ui_theme_source_tests {
    use std::hint::black_box;
    use std::path::PathBuf;
    use std::time::Instant;

    use super::*;
    use crate::asset::AssetUri;

    const SAMPLE_PAIRS: usize = 21;
    const CHECKS_PER_SAMPLE: usize = 16;
    const SOURCE_BYTES: usize = 1_048_576;

    #[test]
    fn borrowed_ui_document_source_contract_theme_import() {
        let context = AssetImportContext::new(
            PathBuf::from("ui/plugins07.theme.toml"),
            AssetUri::parse("res://ui/plugins07.theme.toml").unwrap(),
            br#"
id = "zircon.plugins07.dark"

[palette]
accent = { red = 0.1, green = 0.2, blue = 0.3, alpha = 1.0 }

[[typography]]
variant = "body"
family = "Inter"
size = 13.0
weight = 400
line_height = 1.45
"#
            .to_vec(),
            toml::Table::new(),
        );

        let outcome = import_ui_theme_asset(&context).unwrap();
        let Some(ImportedAsset::UiTheme(theme)) = outcome.root_entry().map(|entry| &entry.asset)
        else {
            panic!("ui theme importer must preserve its typed root asset")
        };
        assert_eq!(theme.document.id, "zircon.plugins07.dark");
        assert_eq!(theme.document.typography.len(), 1);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn borrowed_ui_document_source_performance_release_theme() {
        run_release_gate("plugins07_borrowed_ui_theme_source");
    }

    fn run_release_gate(marker: &str) {
        let context = benchmark_context();
        for _ in 0..4 {
            black_box(measure_owned(&context));
            black_box(measure_borrowed(&context));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
                (measure_owned(&context), measure_borrowed(&context))
            } else {
                let optimized_ns = measure_borrowed(&context);
                (measure_owned(&context), optimized_ns)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT {marker} sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} source_bytes={SOURCE_BYTES} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=40 legacy_source_string_allocations_per_sample={CHECKS_PER_SAMPLE} optimized_source_string_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            improvement_percent >= 40,
            "borrowed ui theme source preparation must improve P95 by at least 40%"
        );
    }

    fn benchmark_context() -> AssetImportContext {
        AssetImportContext::new(
            PathBuf::from("ui/plugins07-large.theme.toml"),
            AssetUri::parse("res://ui/plugins07-large.theme.toml").unwrap(),
            vec![b'a'; SOURCE_BYTES],
            toml::Table::new(),
        )
    }

    fn measure_owned(context: &AssetImportContext) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            black_box(black_box(context).source_text().unwrap());
        }
        started.elapsed().as_nanos().max(1)
    }

    fn measure_borrowed(context: &AssetImportContext) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            black_box(black_box(context).source_str().unwrap());
        }
        started.elapsed().as_nanos().max(1)
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
