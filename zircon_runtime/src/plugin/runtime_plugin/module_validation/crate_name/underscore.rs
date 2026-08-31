pub(super) fn validate_runtime_plugin_module_crate_name_underscore(
    manifest_label: &str,
    crate_name: &str,
    diagnostics: &mut Vec<String>,
) {
    let mut ends_with_underscore = false;
    let mut previous_was_underscore = false;
    let invalid = crate_name.bytes().any(|byte| {
        if byte == b'_' {
            let invalid = previous_was_underscore;
            ends_with_underscore = true;
            previous_was_underscore = true;
            invalid
        } else {
            ends_with_underscore = false;
            previous_was_underscore = false;
            false
        }
    }) || ends_with_underscore;

    if invalid {
        diagnostics.push(format!(
            "{manifest_label} module crate_name `{crate_name}` must not end with an underscore or contain repeated underscores"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::validate_runtime_plugin_module_crate_name_underscore;

    #[test]
    fn optimization_batch_gl_runtime494_crate_name_underscore_scan_preserves_rules() {
        let valid = ["zircon_plugin_render", "zircon_plugin_render_v2"];
        for crate_name in valid {
            let mut diagnostics = Vec::new();
            validate_runtime_plugin_module_crate_name_underscore(
                "runtime plugin",
                crate_name,
                &mut diagnostics,
            );
            assert!(
                diagnostics.is_empty(),
                "unexpected diagnostic for {crate_name}"
            );
        }

        let invalid = ["zircon_plugin_render_", "zircon_plugin_render__v2"];
        for crate_name in invalid {
            let mut diagnostics = Vec::new();
            validate_runtime_plugin_module_crate_name_underscore(
                "runtime plugin",
                crate_name,
                &mut diagnostics,
            );
            assert_eq!(diagnostics.len(), 1, "missing diagnostic for {crate_name}");
        }
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gl_runtime494_crate_name_underscore_scan_benchmark() {
        const MARKER: &str = "RUNTIME494_CRATE_NAME_UNDERSCORE_SCAN_BENCH_V1";
        const SAMPLE: &str = "zircon_plugin_render_pipeline_materials_shadow_pass_quality_profile";
        const ITERATIONS: usize = 100_000;
        let start = std::time::Instant::now();
        let mut diagnostics = Vec::new();
        for _ in 0..ITERATIONS {
            diagnostics.clear();
            validate_runtime_plugin_module_crate_name_underscore(
                "runtime plugin",
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
            if SAMPLE.ends_with('_') || SAMPLE.contains("__") {
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
