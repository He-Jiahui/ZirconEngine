pub(super) fn validate_runtime_plugin_package_root_relative(
    field_name: &str,
    root: &str,
    diagnostics: &mut Vec<String>,
) {
    if root
        .as_bytes()
        .first()
        .is_some_and(|byte| matches!(byte, b'/' | b'\\'))
    {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} root `{root}` must be relative"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::validate_runtime_plugin_package_root_relative;

    #[test]
    fn optimization_batch_gk_runtime493_root_relative_prefix_preserves_rules() {
        let valid = ["assets/materials", "plugins/render", "./local"];
        for root in valid {
            let mut diagnostics = Vec::new();
            validate_runtime_plugin_package_root_relative("root", root, &mut diagnostics);
            assert!(diagnostics.is_empty(), "unexpected diagnostic for {root}");
        }

        let invalid = ["/assets/materials", "\\assets\\materials"];
        for root in invalid {
            let mut diagnostics = Vec::new();
            validate_runtime_plugin_package_root_relative("root", root, &mut diagnostics);
            assert_eq!(diagnostics.len(), 1, "missing diagnostic for {root}");
        }
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gk_runtime493_root_relative_prefix_benchmark() {
        const MARKER: &str = "RUNTIME493_ROOT_RELATIVE_PREFIX_BENCH_V1";
        const SAMPLE: &str =
            "assets/materials/shaders/forward_plus/clustered_lighting/quality_profile";
        const ITERATIONS: usize = 100_000;
        let start = std::time::Instant::now();
        let mut diagnostics = Vec::new();
        for _ in 0..ITERATIONS {
            diagnostics.clear();
            validate_runtime_plugin_package_root_relative("root", SAMPLE, &mut diagnostics);
            assert!(diagnostics.is_empty());
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        let mut legacy_diagnostics = Vec::new();
        for _ in 0..ITERATIONS {
            legacy_diagnostics.clear();
            if SAMPLE.starts_with('/') || SAMPLE.starts_with('\\') {
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
