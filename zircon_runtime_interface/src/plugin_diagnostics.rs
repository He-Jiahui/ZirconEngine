use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistrationDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrationDiagnostic {
    pub severity: RegistrationDiagnosticSeverity,
    pub code: String,
    pub plugin_id: String,
    pub message: String,
}

impl RegistrationDiagnostic {
    pub fn new(
        severity: RegistrationDiagnosticSeverity,
        code: impl Into<String>,
        plugin_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code: code.into(),
            plugin_id: plugin_id.into(),
            message: message.into(),
        }
    }

    pub fn missing_capability(plugin_id: impl Into<String>, capability: impl Into<String>) -> Self {
        let plugin_id = plugin_id.into();
        let capability = capability.into();
        let mut message = String::with_capacity(
            "editor plugin `` requires missing capability ``"
                .len()
                .saturating_add(plugin_id.len())
                .saturating_add(capability.len()),
        );
        message.push_str("editor plugin `");
        message.push_str(&plugin_id);
        message.push_str("` requires missing capability `");
        message.push_str(&capability);
        message.push('`');
        Self::new(
            RegistrationDiagnosticSeverity::Error,
            "editor.capability.missing",
            plugin_id,
            message,
        )
    }

    pub fn is_error(&self) -> bool {
        self.severity == RegistrationDiagnosticSeverity::Error
    }
}

#[cfg(test)]
mod tests {
    use std::{hint::black_box, time::Instant};

    use super::{RegistrationDiagnostic, RegistrationDiagnosticSeverity};

    const PLUGIN_ID_BYTES: usize = 16_384;
    const SAMPLE_PAIRS: usize = 21;

    fn legacy_missing_capability(plugin_id: String, capability: String) -> RegistrationDiagnostic {
        RegistrationDiagnostic::new(
            RegistrationDiagnosticSeverity::Error,
            "editor.capability.missing",
            plugin_id.clone(),
            format!("editor plugin `{plugin_id}` requires missing capability `{capability}`"),
        )
    }

    fn p95(mut samples: Vec<u128>) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100) - 1]
    }

    fn measure<T>(run: impl FnOnce() -> T) -> (u128, T) {
        let start = Instant::now();
        let output = run();
        (start.elapsed().as_nanos(), output)
    }

    #[test]
    fn missing_capability_preserves_fields_and_message() {
        let diagnostic =
            RegistrationDiagnostic::missing_capability("sample.plugin", "asset.import");

        assert_eq!(diagnostic.severity, RegistrationDiagnosticSeverity::Error);
        assert_eq!(diagnostic.code, "editor.capability.missing");
        assert_eq!(diagnostic.plugin_id, "sample.plugin");
        assert_eq!(
            diagnostic.message,
            "editor plugin `sample.plugin` requires missing capability `asset.import`"
        );
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn missing_capability_moves_owned_plugin_id_without_clone() {
        let plugin_id = "plugin.".to_owned() + &"x".repeat(PLUGIN_ID_BYTES - 7);
        let capability = "asset.import".to_owned();

        for _ in 0..5 {
            black_box(legacy_missing_capability(
                plugin_id.clone(),
                capability.clone(),
            ));
            black_box(RegistrationDiagnostic::missing_capability(
                plugin_id.clone(),
                capability.clone(),
            ));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            let legacy_plugin_id = plugin_id.clone();
            let legacy_capability = capability.clone();
            let optimized_plugin_id = plugin_id.clone();
            let optimized_capability = capability.clone();
            if sample % 2 == 0 {
                let (elapsed, diagnostic) =
                    measure(|| legacy_missing_capability(legacy_plugin_id, legacy_capability));
                black_box(diagnostic);
                legacy_samples.push(elapsed);
                let (elapsed, diagnostic) = measure(|| {
                    RegistrationDiagnostic::missing_capability(
                        optimized_plugin_id,
                        optimized_capability,
                    )
                });
                black_box(diagnostic);
                optimized_samples.push(elapsed);
            } else {
                let (elapsed, diagnostic) = measure(|| {
                    RegistrationDiagnostic::missing_capability(
                        optimized_plugin_id,
                        optimized_capability,
                    )
                });
                black_box(diagnostic);
                optimized_samples.push(elapsed);
                let (elapsed, diagnostic) =
                    measure(|| legacy_missing_capability(legacy_plugin_id, legacy_capability));
                black_box(diagnostic);
                legacy_samples.push(elapsed);
            }
        }

        let legacy_p95_ns = p95(legacy_samples);
        let optimized_p95_ns = p95(optimized_samples);
        println!(
            "PERF_RESULT runtime_interface04_registration_diagnostic \
             plugin_id_bytes={PLUGIN_ID_BYTES} sample_pairs={SAMPLE_PAIRS} \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             legacy_plugin_id_clones=1 optimized_plugin_id_clones=0"
        );
        assert!(
            optimized_p95_ns * 100 <= legacy_p95_ns * 80,
            "clone-free diagnostic construction must be <=80% of legacy P95: \
             optimized={optimized_p95_ns}ns legacy={legacy_p95_ns}ns"
        );
    }
}
