use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, DataAsset, DataAssetFormat,
    ImportedAsset,
};

pub(crate) fn import_plain_toml_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_str()?;
    let value: toml::Value =
        toml::from_str(source).map_err(|source| AssetImportError::TomlDeserialize {
            context: "parsing TOML data asset",
            source,
        })?;
    let canonical_json = serde_json::to_value(value)?;
    let text = source.to_owned();
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Toml,
            text,
            canonical_json,
        }),
    ))
}

pub(crate) fn import_json_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_str()?;
    let canonical_json =
        serde_json::from_str(source).map_err(|source| AssetImportError::JsonDeserialize {
            context: "parsing JSON data asset",
            source,
        })?;
    let text = source.to_owned();
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Json,
            text,
            canonical_json,
        }),
    ))
}

pub(crate) fn import_text_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Text,
            text,
            canonical_json: serde_json::Value::Null,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const BENCHMARK_SOURCE_BYTES: usize = 1_048_576;
    const BENCHMARK_CHECKS: usize = 16;
    const BENCHMARK_SAMPLE_PAIRS: usize = 21;
    const BENCHMARK_THRESHOLD_PERCENT: u128 = 40;

    fn imported_data(outcome: &AssetImportOutcome) -> &DataAsset {
        match &outcome.root_entry().expect("data root entry").asset {
            ImportedAsset::Data(asset) => asset,
            other => panic!("expected data asset, got {other:?}"),
        }
    }

    fn legacy_import_plain_toml_data(context: &AssetImportContext) -> Result<(), AssetImportError> {
        let text = context.source_text()?;
        let _: toml::Value =
            toml::from_str(&text).map_err(|source| AssetImportError::TomlDeserialize {
                context: "parsing TOML data asset",
                source,
            })?;
        Ok(())
    }

    fn legacy_import_json_data(context: &AssetImportContext) -> Result<(), AssetImportError> {
        let text = context.source_text()?;
        let _: serde_json::Value =
            serde_json::from_str(&text).map_err(|source| AssetImportError::JsonDeserialize {
                context: "parsing JSON data asset",
                source,
            })?;
        Ok(())
    }

    fn measure_rejected_import(
        iterations: usize,
        mut import: impl FnMut() -> AssetImportError,
    ) -> u128 {
        let timer = Instant::now();
        for _ in 0..iterations {
            black_box(import());
        }
        timer.elapsed().as_nanos()
    }

    fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95 - 1) / 100]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    #[test]
    fn borrowed_toml_parser_preserves_successful_source() {
        let source = "# retained comment\nvalue = 7\n";
        let context = data_context("valid.toml", source.as_bytes().to_vec());

        let outcome = import_plain_toml_data(&context).expect("valid TOML data import");

        assert_eq!(imported_data(&outcome).text, source);
        assert_eq!(imported_data(&outcome).canonical_json["value"], 7);
    }

    #[test]
    fn borrowed_json_parser_preserves_successful_source() {
        let source = "{\n  \"value\": 7\n}\n";
        let context = data_context("valid.json", source.as_bytes().to_vec());

        let outcome = import_json_data(&context).expect("valid JSON data import");

        assert_eq!(imported_data(&outcome).text, source);
        assert_eq!(imported_data(&outcome).canonical_json["value"], 7);
    }

    #[test]
    fn source_text_decode_error_retains_utf8_source() {
        let context = data_context("invalid.txt", vec![0xff]);

        let error = import_error(import_text_data(&context));

        match &error {
            AssetImportError::SourceTextDecode { path, .. } => {
                assert_eq!(path, &std::path::PathBuf::from("invalid.txt"));
            }
            other => panic!("expected source text decode error, got {other:?}"),
        }
        assert!(error.source().is_some());
    }

    #[test]
    fn toml_data_parse_error_retains_toml_source() {
        let context = data_context("invalid.toml", b"value = [".to_vec());

        let error = import_error(import_plain_toml_data(&context));

        match &error {
            AssetImportError::TomlDeserialize { context, .. } => {
                assert_eq!(*context, "parsing TOML data asset");
            }
            other => panic!("expected TOML deserialize error, got {other:?}"),
        }
        assert!(error.source().is_some());
    }

    #[test]
    fn json_data_parse_error_retains_json_source() {
        let context = data_context("invalid.json", b"{".to_vec());

        let error = import_error(import_json_data(&context));

        match &error {
            AssetImportError::JsonDeserialize { context, .. } => {
                assert_eq!(*context, "parsing JSON data asset");
            }
            other => panic!("expected JSON deserialize error, got {other:?}"),
        }
        assert!(error.source().is_some());
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn benchmark_borrowed_toml_failure_source() {
        let mut source = vec![b' '; BENCHMARK_SOURCE_BYTES];
        source[0] = 0;
        let context = data_context("large-invalid.toml", source);
        assert!(matches!(
            import_plain_toml_data(&context),
            Err(AssetImportError::TomlDeserialize { .. })
        ));

        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        for sample_index in 0..BENCHMARK_SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_rejected_import(BENCHMARK_CHECKS, || {
                    legacy_import_plain_toml_data(black_box(&context)).unwrap_err()
                }));
                optimized_samples.push(measure_rejected_import(BENCHMARK_CHECKS, || {
                    import_plain_toml_data(black_box(&context)).unwrap_err()
                }));
            } else {
                optimized_samples.push(measure_rejected_import(BENCHMARK_CHECKS, || {
                    import_plain_toml_data(black_box(&context)).unwrap_err()
                }));
                legacy_samples.push(measure_rejected_import(BENCHMARK_CHECKS, || {
                    legacy_import_plain_toml_data(black_box(&context)).unwrap_err()
                }));
            }
        }

        print_failure_benchmark(
            "plugins07_borrowed_toml_failure_source",
            &legacy_samples,
            &optimized_samples,
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn benchmark_borrowed_json_failure_source() {
        let mut source = vec![b' '; BENCHMARK_SOURCE_BYTES];
        source[0] = b']';
        let context = data_context("large-invalid.json", source);
        assert!(matches!(
            import_json_data(&context),
            Err(AssetImportError::JsonDeserialize { .. })
        ));

        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        for sample_index in 0..BENCHMARK_SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_rejected_import(BENCHMARK_CHECKS, || {
                    legacy_import_json_data(black_box(&context)).unwrap_err()
                }));
                optimized_samples.push(measure_rejected_import(BENCHMARK_CHECKS, || {
                    import_json_data(black_box(&context)).unwrap_err()
                }));
            } else {
                optimized_samples.push(measure_rejected_import(BENCHMARK_CHECKS, || {
                    import_json_data(black_box(&context)).unwrap_err()
                }));
                legacy_samples.push(measure_rejected_import(BENCHMARK_CHECKS, || {
                    legacy_import_json_data(black_box(&context)).unwrap_err()
                }));
            }
        }

        print_failure_benchmark(
            "plugins07_borrowed_json_failure_source",
            &legacy_samples,
            &optimized_samples,
        );
    }

    fn print_failure_benchmark(marker: &str, legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_raw = legacy_samples.to_vec();
        let optimized_raw = optimized_samples.to_vec();
        let mut legacy_sorted = legacy_raw.clone();
        let mut optimized_sorted = optimized_raw.clone();
        let legacy_p95_ns = nearest_rank_p95(&mut legacy_sorted);
        let optimized_p95_ns = nearest_rank_p95(&mut optimized_sorted);
        let improvement_percent = legacy_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(100)
            / legacy_p95_ns.max(1);

        println!(
            "PERF_RESULT {marker} source_bytes={} checks_per_sample={} sample_pairs={} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_source_string_allocations_per_sample={} optimized_source_string_allocations_per_sample=0 legacy_source_bytes_copied_per_sample={} optimized_source_bytes_copied_per_sample=0 legacy_p95_ns={} optimized_p95_ns={} improvement_percent={} threshold_percent={} legacy_ns={} optimized_ns={}",
            BENCHMARK_SOURCE_BYTES,
            BENCHMARK_CHECKS,
            BENCHMARK_SAMPLE_PAIRS,
            BENCHMARK_CHECKS,
            BENCHMARK_SOURCE_BYTES * BENCHMARK_CHECKS,
            legacy_p95_ns,
            optimized_p95_ns,
            improvement_percent,
            BENCHMARK_THRESHOLD_PERCENT,
            sample_csv(&legacy_raw),
            sample_csv(&optimized_raw),
        );

        assert_eq!(BENCHMARK_SAMPLE_PAIRS, legacy_raw.len());
        assert_eq!(BENCHMARK_SAMPLE_PAIRS, optimized_raw.len());
        assert!(
            improvement_percent >= BENCHMARK_THRESHOLD_PERCENT,
            "{marker} P95 improvement {improvement_percent}% misses {BENCHMARK_THRESHOLD_PERCENT}% gate"
        );
    }

    fn data_context(path: &str, source_bytes: Vec<u8>) -> AssetImportContext {
        AssetImportContext::new(
            path.into(),
            crate::asset::AssetUri::parse(&format!("res://data/{path}")).unwrap(),
            source_bytes,
            toml::Table::new(),
        )
    }

    fn import_error(result: Result<AssetImportOutcome, AssetImportError>) -> AssetImportError {
        match result {
            Ok(_) => panic!("invalid data import unexpectedly succeeded"),
            Err(error) => error,
        }
    }
}
