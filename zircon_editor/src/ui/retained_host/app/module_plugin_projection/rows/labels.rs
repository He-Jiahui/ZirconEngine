use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;

pub(in crate::ui::retained_host::app::module_plugin_projection) fn module_plugin_primary_action(
    plugin_id: &str,
    enabled: bool,
    required: bool,
) -> (String, String) {
    if required {
        return ("Required".to_string(), String::new());
    }

    if enabled {
        (
            "Disable".to_string(),
            module_plugin_action_id("workbench.plugin.disable", plugin_id),
        )
    } else {
        (
            "Enable".to_string(),
            module_plugin_action_id("workbench.plugin.enable", plugin_id),
        )
    }
}

pub(in crate::ui::retained_host::app::module_plugin_projection) fn module_plugin_action_id(
    prefix: &str,
    plugin_id: &str,
) -> String {
    let mut action_id = String::with_capacity(prefix.len() + 1 + plugin_id.len());
    action_id.push_str(prefix);
    action_id.push('.');
    action_id.push_str(plugin_id);
    action_id
}

pub(in crate::ui::retained_host::app::module_plugin_projection) fn target_mode_label(
    mode: &RuntimeTargetMode,
) -> &'static str {
    match mode {
        RuntimeTargetMode::ClientRuntime => "client",
        RuntimeTargetMode::ServerRuntime => "server",
        RuntimeTargetMode::EditorHost => "editor",
    }
}

pub(in crate::ui::retained_host::app::module_plugin_projection) fn packaging_label(
    strategy: ExportPackagingStrategy,
) -> &'static str {
    match strategy {
        ExportPackagingStrategy::SourceTemplate => "source-template",
        ExportPackagingStrategy::LibraryEmbed => "library-embed",
        ExportPackagingStrategy::NativeDynamic => "native-dynamic",
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const IDS_PER_SAMPLE: usize = 262_144;

    #[test]
    fn optimization_batch_ew_editor385_preserves_module_plugin_action_ids() {
        for (prefix, plugin_id) in [
            ("workbench.plugin.enable", "rendering"),
            ("workbench.plugin.hot_reload", "org.zircon.weather"),
            ("", "plugin"),
            ("workbench.plugin.disable", ""),
        ] {
            assert_eq!(
                module_plugin_action_id(prefix, plugin_id),
                format!("{prefix}.{plugin_id}")
            );
        }

        let production = include_str!("labels.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        assert!(!production.contains("format!("));
        assert!(production.contains("String::with_capacity(prefix.len() + 1 + plugin_id.len())"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_ew_editor385_direct_module_plugin_action_id_benchmark() {
        const PREFIX: &str = "workbench.plugin.target_modes.next";
        const PLUGIN_ID: &str = "org.zircon.rendering.deferred";

        for _ in 0..4 {
            black_box(measure_legacy_ids(PREFIX, PLUGIN_ID));
            black_box(measure_direct_ids(PREFIX, PLUGIN_ID));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy_ids(PREFIX, PLUGIN_ID));
                optimized_samples.push(measure_direct_ids(PREFIX, PLUGIN_ID));
            } else {
                optimized_samples.push(measure_direct_ids(PREFIX, PLUGIN_ID));
                legacy_samples.push(measure_legacy_ids(PREFIX, PLUGIN_ID));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn measure_legacy_ids(prefix: &str, plugin_id: &str) -> u128 {
        measure_ids(prefix, plugin_id, |prefix, plugin_id| {
            format!("{prefix}.{plugin_id}")
        })
    }

    fn measure_direct_ids(prefix: &str, plugin_id: &str) -> u128 {
        measure_ids(prefix, plugin_id, module_plugin_action_id)
    }

    fn measure_ids(
        prefix: &str,
        plugin_id: &str,
        mut build: impl FnMut(&str, &str) -> String,
    ) -> u128 {
        let started = Instant::now();
        let mut total_len = 0_usize;
        for _ in 0..IDS_PER_SAMPLE {
            let action_id = build(black_box(prefix), black_box(plugin_id));
            total_len += black_box(action_id.len());
            black_box(action_id);
        }
        assert_eq!(
            black_box(total_len),
            IDS_PER_SAMPLE * (prefix.len() + 1 + plugin_id.len())
        );
        started.elapsed().as_nanos().max(1)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR385_DIRECT_MODULE_PLUGIN_ACTION_ID_BENCH_V1 sample_pairs={SAMPLE_PAIRS} ids_per_sample={IDS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=35",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(65) / 100,
            "direct module plugin action id construction must reduce P95 by at least 35%"
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
