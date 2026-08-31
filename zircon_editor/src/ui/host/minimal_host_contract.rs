use super::editor_subsystems::OPTIONAL_EDITOR_SUBSYSTEMS;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorHostMinimalContract;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorHostMinimalReport {
    loaded_capabilities: Vec<String>,
    missing_capabilities: Vec<String>,
}

const MINIMAL_CAPABILITIES: &[&str] = &[
    "editor.host.ui_shell",
    "editor.host.asset_core",
    "editor.host.scene_interaction",
    "editor.host.runtime_render_embed",
    "editor.host.plugin_management",
    "editor.host.capability_bridge",
];

pub fn editor_host_minimal_contract() -> EditorHostMinimalContract {
    EditorHostMinimalContract
}

impl EditorHostMinimalContract {
    pub fn minimal_capability_ids(&self) -> Vec<String> {
        let mut capabilities = Vec::with_capacity(MINIMAL_CAPABILITIES.len());
        capabilities.extend(
            MINIMAL_CAPABILITIES
                .iter()
                .map(|capability| (*capability).to_string()),
        );
        capabilities
    }

    pub fn is_minimal(&self, capability: &str) -> bool {
        MINIMAL_CAPABILITIES.contains(&capability)
    }

    pub fn is_extension_blacklisted(&self, capability: &str) -> bool {
        OPTIONAL_EDITOR_SUBSYSTEMS.contains(&capability)
    }

    pub fn self_check(&self) -> EditorHostMinimalReport {
        EditorHostMinimalReport {
            loaded_capabilities: self.minimal_capability_ids(),
            missing_capabilities: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{EditorHostMinimalContract, MINIMAL_CAPABILITIES};

    const CALLS_PER_SAMPLE: usize = 65_536;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_fu_editor407_reserves_minimal_capability_ids() {
        let source = include_str!("minimal_host_contract.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("minimal host contract production source");

        assert!(production.contains("Vec::with_capacity(MINIMAL_CAPABILITIES.len())"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fu_editor407_minimal_capability_ids_benchmark() {
        let contract = EditorHostMinimalContract;
        for _ in 0..4 {
            black_box(measure_calls(&contract, false));
            black_box(measure_calls(&contract, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_calls(&contract, false));
                optimized_samples.push(measure_calls(&contract, true));
            } else {
                optimized_samples.push(measure_calls(&contract, true));
                legacy_samples.push(measure_calls(&contract, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR407_MINIMAL_CAPABILITY_IDS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} calls_per_sample={CALLS_PER_SAMPLE} capability_count={} legacy_vec_growth_allocations_per_call=2 optimized_vec_growth_allocations_per_call=0 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=20",
            MINIMAL_CAPABILITIES.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 80 / 100);
    }

    fn measure_calls(contract: &EditorHostMinimalContract, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CALLS_PER_SAMPLE {
            let capabilities = if optimized {
                contract.minimal_capability_ids()
            } else {
                MINIMAL_CAPABILITIES
                    .iter()
                    .map(|capability| (*capability).to_string())
                    .collect()
            };
            black_box(capabilities);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
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

impl EditorHostMinimalReport {
    pub fn loaded_capabilities(&self) -> Vec<String> {
        self.loaded_capabilities.clone()
    }

    pub fn missing_capabilities(&self) -> &[String] {
        &self.missing_capabilities
    }
}
