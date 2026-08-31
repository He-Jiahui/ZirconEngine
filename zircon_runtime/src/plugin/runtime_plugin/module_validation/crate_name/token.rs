use super::super::super::package_validation::is_lowercase_runtime_plugin_token;

pub(super) fn validate_runtime_plugin_module_crate_name_token(
    manifest_label: &str,
    crate_name: &str,
    diagnostics: &mut Vec<String>,
) {
    const PREFIX: &[u8] = b"zircon_plugin_";
    let bytes = crate_name.as_bytes();
    let valid = bytes.len() >= PREFIX.len()
        && bytes.starts_with(PREFIX)
        && bytes[PREFIX.len()..]
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_');

    if !valid {
        diagnostics.push(format!(
            "{manifest_label} module crate_name `{crate_name}` must use `zircon_plugin_` prefix and contain only lowercase ASCII letters, digits, and underscores"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_lowercase_runtime_plugin_token, validate_runtime_plugin_module_crate_name_token,
    };

    #[test]
    fn optimization_batch_go_runtime497_crate_name_token_scan_preserves_rules() {
        let valid = ["zircon_plugin_render", "zircon_plugin_render_v2"];
        for crate_name in valid {
            let mut diagnostics = Vec::new();
            validate_runtime_plugin_module_crate_name_token(
                "runtime plugin",
                crate_name,
                &mut diagnostics,
            );
            assert!(
                diagnostics.is_empty(),
                "unexpected diagnostic for {crate_name}"
            );
        }

        let invalid = [
            "plugin_render",
            "zircon_plugin_Render",
            "zircon_plugin_render-",
        ];
        for crate_name in invalid {
            let mut diagnostics = Vec::new();
            validate_runtime_plugin_module_crate_name_token(
                "runtime plugin",
                crate_name,
                &mut diagnostics,
            );
            assert_eq!(diagnostics.len(), 1, "missing diagnostic for {crate_name}");
        }
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_go_runtime497_crate_name_token_scan_benchmark() {
        const MARKER: &str = "RUNTIME497_CRATE_NAME_TOKEN_SCAN_BENCH_V1";
        const SAMPLE: &str = "zircon_plugin_render_pipeline_materials_shadow_pass_quality_profile";
        const ITERATIONS: usize = 100_000;
        let start = std::time::Instant::now();
        let mut diagnostics = Vec::new();
        for _ in 0..ITERATIONS {
            diagnostics.clear();
            validate_runtime_plugin_module_crate_name_token(
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
            if !SAMPLE.starts_with("zircon_plugin_") || !is_lowercase_runtime_plugin_token(SAMPLE) {
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
