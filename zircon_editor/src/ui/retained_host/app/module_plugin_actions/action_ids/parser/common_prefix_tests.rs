use std::hint::black_box;
use std::time::Instant;

use super::{parse_module_plugin_action, ModulePluginAction};

const ACTION_BYTES: usize = 512;
const CHECKS_PER_SAMPLE: usize = 131_072;
const SAMPLE_PAIRS: usize = 31;

fn legacy_parse_module_plugin_action(action_id: &str) -> Option<ModulePluginAction<'_>> {
    action_id
        .strip_prefix("workbench.plugin.enable.")
        .map(|plugin_id| ModulePluginAction::SetEnabled {
            plugin_id,
            enabled: true,
        })
        .or_else(|| {
            action_id
                .strip_prefix("workbench.plugin.disable.")
                .map(|plugin_id| ModulePluginAction::SetEnabled {
                    plugin_id,
                    enabled: false,
                })
        })
        .or_else(|| {
            action_id
                .strip_prefix("workbench.plugin.packaging.next.")
                .map(|plugin_id| ModulePluginAction::CyclePackaging { plugin_id })
        })
        .or_else(|| {
            action_id
                .strip_prefix("workbench.plugin.target_modes.next.")
                .map(|plugin_id| ModulePluginAction::CycleTargetModes { plugin_id })
        })
        .or_else(|| {
            action_id
                .strip_prefix("workbench.plugin.feature.enable_dependencies.")
                .and_then(super::parse_module_plugin_feature_action)
                .map(
                    |(plugin_id, feature_id)| ModulePluginAction::EnableFeatureDependencies {
                        plugin_id,
                        feature_id,
                    },
                )
        })
        .or_else(|| {
            action_id
                .strip_prefix("workbench.plugin.feature.enable.")
                .and_then(super::parse_module_plugin_feature_action)
                .map(
                    |(plugin_id, feature_id)| ModulePluginAction::SetFeatureEnabled {
                        plugin_id,
                        feature_id,
                        enabled: true,
                    },
                )
        })
        .or_else(|| {
            action_id
                .strip_prefix("workbench.plugin.feature.disable.")
                .and_then(super::parse_module_plugin_feature_action)
                .map(
                    |(plugin_id, feature_id)| ModulePluginAction::SetFeatureEnabled {
                        plugin_id,
                        feature_id,
                        enabled: false,
                    },
                )
        })
        .or_else(|| {
            action_id
                .strip_prefix("workbench.plugin.unload.")
                .map(|plugin_id| ModulePluginAction::Unload { plugin_id })
        })
        .or_else(|| {
            action_id
                .strip_prefix("workbench.plugin.hot_reload.")
                .map(|plugin_id| ModulePluginAction::HotReload { plugin_id })
        })
}

fn measure(action_id: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut matches = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        let action = if optimized {
            parse_module_plugin_action(black_box(action_id))
        } else {
            legacy_parse_module_plugin_action(black_box(action_id))
        };
        matches += usize::from(matches!(action, Some(ModulePluginAction::HotReload { .. })));
    }
    black_box(matches);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn optimization_batch_20260829bc_editor275_common_prefix_parser_preserves_actions() {
    for action_id in [
        "workbench.plugin.enable.render",
        "workbench.plugin.disable.render",
        "workbench.plugin.packaging.next.render",
        "workbench.plugin.target_modes.next.render",
        "workbench.plugin.feature.enable_dependencies.render.shadow",
        "workbench.plugin.feature.enable.render.shadow",
        "workbench.plugin.feature.disable.render.shadow",
        "workbench.plugin.unload.render",
        "workbench.plugin.hot_reload.render",
        "workbench.plugin.feature.enable.render",
        "workbench.other.enable.render",
    ] {
        assert_eq!(
            parse_module_plugin_action(action_id),
            legacy_parse_module_plugin_action(action_id),
            "{action_id:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bc_editor275_module_plugin_parser_strips_common_prefix_once() {
    let source = include_str!("../parser.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert_eq!(production.matches("workbench.plugin.").count(), 1);
    assert!(production.contains("let action = action_id.strip_prefix"));
    assert!(production.contains("action\n                .strip_prefix(\"hot_reload.\")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bc_editor275_common_prefix_module_plugin_actions_bench() {
    let plugin_bytes = ACTION_BYTES - "workbench.plugin.hot_reload.".len();
    let action_id = format!("workbench.plugin.hot_reload.{}", "p".repeat(plugin_bytes));
    assert_eq!(action_id.len(), ACTION_BYTES);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&action_id, false));
            optimized_samples.push(measure(&action_id, true));
        } else {
            optimized_samples.push(measure(&action_id, true));
            legacy_samples.push(measure(&action_id, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR275_COMMON_PREFIX_MODULE_PLUGIN_ACTIONS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} action_bytes={ACTION_BYTES} \
legacy_common_prefix_checks=9 optimized_common_prefix_checks=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}
