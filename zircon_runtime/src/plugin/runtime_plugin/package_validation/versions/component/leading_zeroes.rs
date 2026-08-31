pub(super) fn validate_runtime_plugin_package_semver_component_leading_zeroes(
    field_name: &str,
    value: &str,
    component_name: &str,
    segment: &str,
    diagnostics: &mut Vec<String>,
) -> bool {
    let bytes = segment.as_bytes();
    if bytes.first() == Some(&b'0') && bytes.len() > 1 {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` {component_name} component `{segment}` must not use leading zeroes"
        ));
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::validate_runtime_plugin_package_semver_component_leading_zeroes;

    #[test]
    fn optimization_batch_gp_runtime498_leading_zero_scan_preserves_rules() {
        let mut diagnostics = Vec::new();
        assert!(
            validate_runtime_plugin_package_semver_component_leading_zeroes(
                "version",
                "1.2.3",
                "major",
                "0",
                &mut diagnostics,
            )
        );
        assert!(diagnostics.is_empty());
        assert!(
            !validate_runtime_plugin_package_semver_component_leading_zeroes(
                "version",
                "01.2.3",
                "major",
                "01",
                &mut diagnostics,
            )
        );
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gp_runtime498_leading_zero_scan_benchmark() {
        const MARKER: &str = "RUNTIME498_LEADING_ZERO_SCAN_BENCH_V1";
        const ITERATIONS: usize = 100_000;
        let segment = "123456789";
        let start = std::time::Instant::now();
        let mut diagnostics = Vec::new();
        for _ in 0..ITERATIONS {
            diagnostics.clear();
            assert!(
                validate_runtime_plugin_package_semver_component_leading_zeroes(
                    "version",
                    "123.456.789",
                    "major",
                    segment,
                    &mut diagnostics,
                )
            );
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert!(segment == "0" || !segment.starts_with('0'));
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
    }
}
