pub(super) fn validate_runtime_plugin_feature_namespace_segment_tokens(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let mut segment_is_non_empty = false;
    let invalid = value.bytes().any(|byte| {
        if byte == b'.' {
            let invalid = !segment_is_non_empty;
            segment_is_non_empty = false;
            invalid
        } else if byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' {
            segment_is_non_empty = true;
            false
        } else {
            true
        }
    }) || !segment_is_non_empty;

    if invalid {
        diagnostics.push(format!(
            "runtime plugin feature manifest {field_name} `{value}` must contain only lowercase ASCII letters, digits, underscores, and dots"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::validate_runtime_plugin_feature_namespace_segment_tokens;

    #[test]
    fn optimization_batch_gf_runtime488_namespace_token_scan_preserves_segment_rules() {
        let valid = ["render", "render.2d", "_private.v2"];
        for value in valid {
            let mut diagnostics = Vec::new();
            validate_runtime_plugin_feature_namespace_segment_tokens(
                "namespace",
                value,
                &mut diagnostics,
            );
            assert!(diagnostics.is_empty(), "unexpected diagnostic for {value}");
        }

        let invalid = [
            "",
            ".render",
            "render.",
            "render..2d",
            "Render",
            "render-2d",
        ];
        for value in invalid {
            let mut diagnostics = Vec::new();
            validate_runtime_plugin_feature_namespace_segment_tokens(
                "namespace",
                value,
                &mut diagnostics,
            );
            assert_eq!(diagnostics.len(), 1, "missing diagnostic for {value}");
        }
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gf_runtime488_namespace_token_scan_benchmark() {
        const MARKER: &str = "RUNTIME488_NAMESPACE_TOKEN_SCAN_BENCH_V1";
        const SAMPLE: &str = "render.pipeline.materials.shadow_pass.v2.alpha_mask.quality_profile";
        const ITERATIONS: usize = 100_000;
        let start = std::time::Instant::now();
        let mut diagnostics = Vec::new();
        for _ in 0..ITERATIONS {
            diagnostics.clear();
            validate_runtime_plugin_feature_namespace_segment_tokens(
                "namespace",
                SAMPLE,
                &mut diagnostics,
            );
            assert!(diagnostics.is_empty());
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        let mut legacy_diagnostics = Vec::new();
        for _ in 0..ITERATIONS {
            legacy_diagnostics.clear();
            if SAMPLE.split('.').any(|segment| {
                segment.is_empty()
                    || segment.bytes().any(|byte| {
                        !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
                    })
            }) {
                legacy_diagnostics.push(SAMPLE);
            }
            assert!(legacy_diagnostics.is_empty());
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.85"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 85 / 100);
    }
}
