pub(super) fn validate_runtime_plugin_package_id_underscore(
    context: &str,
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let mut segment_ends_with_underscore = false;
    let mut previous_was_underscore = false;
    let invalid = value.bytes().any(|byte| {
        if byte == b'.' {
            let invalid = segment_ends_with_underscore;
            segment_ends_with_underscore = false;
            previous_was_underscore = false;
            invalid
        } else if byte == b'_' {
            let invalid = previous_was_underscore;
            segment_ends_with_underscore = true;
            previous_was_underscore = true;
            invalid
        } else {
            segment_ends_with_underscore = false;
            previous_was_underscore = false;
            false
        }
    }) || segment_ends_with_underscore;

    if invalid {
        diagnostics.push(format!(
            "{context} {field_name} `{value}` segments must not end with an underscore or contain repeated underscores"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::validate_runtime_plugin_package_id_underscore;

    #[test]
    fn optimization_batch_gg_runtime490_underscore_scan_preserves_segment_rules() {
        let valid = ["render.pipeline", "render_v2.pipeline2", "render._private"];
        for value in valid {
            let mut diagnostics = Vec::new();
            validate_runtime_plugin_package_id_underscore(
                "runtime plugin package",
                "package_id",
                value,
                &mut diagnostics,
            );
            assert!(diagnostics.is_empty(), "unexpected diagnostic for {value}");
        }

        let invalid = [
            "render_",
            "render__pipeline",
            "render._private_",
            "render.a__b",
        ];
        for value in invalid {
            let mut diagnostics = Vec::new();
            validate_runtime_plugin_package_id_underscore(
                "runtime plugin package",
                "package_id",
                value,
                &mut diagnostics,
            );
            assert_eq!(diagnostics.len(), 1, "missing diagnostic for {value}");
        }
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gg_runtime490_underscore_scan_benchmark() {
        const MARKER: &str = "RUNTIME490_UNDERSCORE_SCAN_BENCH_V1";
        const SAMPLE: &str = "render.pipeline.materials.shadow_pass.v2.alpha_mask.quality_profile";
        const ITERATIONS: usize = 100_000;
        let start = std::time::Instant::now();
        let mut diagnostics = Vec::new();
        for _ in 0..ITERATIONS {
            diagnostics.clear();
            validate_runtime_plugin_package_id_underscore(
                "runtime plugin package",
                "package_id",
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
            if SAMPLE
                .split('.')
                .any(|segment| segment.ends_with('_') || segment.contains("__"))
            {
                legacy_diagnostics.push(SAMPLE);
            }
            assert!(legacy_diagnostics.is_empty());
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
    }
}
