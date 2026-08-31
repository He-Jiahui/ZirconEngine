use crate::core::framework::bridge::PluginInterface;

use super::{ScriptHostError, ScriptHostValue};

pub const SCRIPT_BEHAVIOR_BRIDGE_INTERFACE_ID: &str = "script.behavior.v1";

/// Stable, provider-qualified asset reference for one script-owned behavior callback.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScriptBehaviorCallbackRef {
    package_id: String,
    node_id: String,
}

impl ScriptBehaviorCallbackRef {
    pub fn new(
        package_id: impl Into<String>,
        node_id: impl Into<String>,
    ) -> Result<Self, ScriptHostError> {
        let package_id = package_id.into();
        let node_id = node_id.into();
        if package_id.is_empty() || package_id.trim() != package_id {
            return Err(ScriptHostError::new(
                "script behavior callback package id must be non-empty and trimmed",
            ));
        }
        if node_id.is_empty() || node_id.trim() != node_id {
            return Err(ScriptHostError::new(
                "script behavior callback node id must be non-empty and trimmed",
            ));
        }
        Ok(Self {
            package_id,
            node_id,
        })
    }

    pub fn parse(value: &str) -> Result<Self, ScriptHostError> {
        let Some((package_id, node_id)) = value.split_once("::") else {
            return Err(ScriptHostError::new(
                "script behavior callback must use `<package>::<node-id>`",
            ));
        };
        Self::new(package_id, node_id)
    }

    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn stable_id(&self) -> String {
        let mut stable_id = String::with_capacity(self.package_id.len() + 2 + self.node_id.len());
        stable_id.push_str(&self.package_id);
        stable_id.push_str("::");
        stable_id.push_str(&self.node_id);
        stable_id
    }
}

/// Neutral call boundary implemented by the script owner and consumed by AI or other plugins.
pub trait ScriptBehaviorBridge: Send + Sync + 'static {
    fn invoke(
        &self,
        callback: &ScriptBehaviorCallbackRef,
        arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, ScriptHostError>;
}

impl PluginInterface for dyn ScriptBehaviorBridge {
    const INTERFACE_ID: &'static str = SCRIPT_BEHAVIOR_BRIDGE_INTERFACE_ID;
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const IDS_PER_SAMPLE: usize = 262_144;

    #[test]
    fn callback_reference_requires_provider_qualified_identity() {
        let callback = ScriptBehaviorCallbackRef::parse("combat::ai.attack").unwrap();
        assert_eq!(callback.package_id(), "combat");
        assert_eq!(callback.node_id(), "ai.attack");
        assert_eq!(callback.stable_id(), "combat::ai.attack");
        assert!(ScriptBehaviorCallbackRef::parse("ai.attack").is_err());
        assert!(ScriptBehaviorCallbackRef::parse("::ai.attack").is_err());
    }

    #[test]
    fn optimization_batch_fb_runtime460_preserves_script_callback_ids() {
        for (package_id, node_id) in [
            ("combat", "ai.attack"),
            ("p", "n"),
            (
                "zircon.gameplay.behavior.runtime",
                "enemy.boss.phase.transition.on_enter",
            ),
        ] {
            let callback = ScriptBehaviorCallbackRef::new(package_id, node_id).unwrap();
            assert_eq!(callback.stable_id(), format!("{package_id}::{node_id}"));
        }
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fb_runtime460_direct_script_callback_id_benchmark() {
        let callback = ScriptBehaviorCallbackRef::new(
            "zircon.gameplay.behavior.runtime",
            "enemy.boss.phase.transition.on_enter",
        )
        .unwrap();
        for _ in 0..4 {
            black_box(measure_legacy(&callback));
            black_box(measure_optimized(&callback));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy(&callback));
                optimized_samples.push(measure_optimized(&callback));
            } else {
                optimized_samples.push(measure_optimized(&callback));
                legacy_samples.push(measure_legacy(&callback));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn measure_legacy(callback: &ScriptBehaviorCallbackRef) -> u128 {
        measure(|| format!("{}::{}", callback.package_id, callback.node_id))
    }

    fn measure_optimized(callback: &ScriptBehaviorCallbackRef) -> u128 {
        measure(|| callback.stable_id())
    }

    fn measure(mut build: impl FnMut() -> String) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..IDS_PER_SAMPLE {
            let value = black_box(build());
            checksum = checksum.wrapping_add(value.len());
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME460_DIRECT_SCRIPT_CALLBACK_ID_BENCH_V1 sample_pairs={SAMPLE_PAIRS} ids_per_sample={IDS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=30",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(70) / 100,
            "direct script callback IDs must reduce P95 by at least 30%"
        );
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
