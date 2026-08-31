use crate::ui::retained_host as host_contract;
use crate::ui::workbench::autolayout::{
    workbench_layout_tier_for_logical_width, WorkbenchLayoutTier,
};

pub(in crate::ui::retained_host::ui) const TABLE_LAYOUT_NARROW_VARIANT: &str = "layoutNarrow";
pub(in crate::ui::retained_host::ui) const TABLE_LAYOUT_REGULAR_VARIANT: &str = "layoutRegular";
pub(in crate::ui::retained_host::ui) const TABLE_LAYOUT_WIDE_VARIANT: &str = "layoutWide";

pub(in crate::ui::retained_host::ui) fn apply_table_layout_context_variant(
    mut node: host_contract::TemplatePaneNodeData,
    context_width: f32,
) -> host_contract::TemplatePaneNodeData {
    if is_table_node(&node) && context_width > 0.0 {
        node.component_variant = append_component_variant_token(
            node.component_variant.as_str(),
            table_layout_context_variant_for_width(context_width),
        )
        .into();
    }
    node
}

pub(in crate::ui::retained_host::ui) fn table_layout_context_variant_for_width(
    context_width: f32,
) -> &'static str {
    match workbench_layout_tier_for_logical_width(context_width) {
        WorkbenchLayoutTier::Ultra | WorkbenchLayoutTier::Narrow => TABLE_LAYOUT_NARROW_VARIANT,
        WorkbenchLayoutTier::Regular => TABLE_LAYOUT_REGULAR_VARIANT,
        WorkbenchLayoutTier::Wide => TABLE_LAYOUT_WIDE_VARIANT,
    }
}

fn is_table_node(node: &host_contract::TemplatePaneNodeData) -> bool {
    node.role.as_str() == "Table" || node.component_role.as_str() == "table"
}

fn append_component_variant_token(variant: &str, token: &str) -> String {
    if token.is_empty() || component_variant_has_token(variant, token) {
        return variant.to_string();
    }
    if variant.trim().is_empty() {
        token.to_string()
    } else {
        let variant = variant.trim();
        let mut combined = String::with_capacity(variant.len() + 1 + token.len());
        combined.push_str(variant);
        combined.push(' ');
        combined.push_str(token);
        combined
    }
}

fn component_variant_has_token(variant: &str, token: &str) -> bool {
    variant
        .split_whitespace()
        .any(|candidate| candidate == token)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const VARIANTS_PER_SAMPLE: usize = 262_144;

    #[test]
    fn optimization_batch_fg_editor393_table_nodes_receive_context_tier_variant() {
        let node = host_contract::TemplatePaneNodeData {
            role: "Table".into(),
            component_role: "table".into(),
            component_variant: "asset-table".into(),
            ..host_contract::TemplatePaneNodeData::default()
        };

        let node = apply_table_layout_context_variant(node, 640.0);

        assert!(node
            .component_variant
            .as_str()
            .split_whitespace()
            .any(|token| token == TABLE_LAYOUT_NARROW_VARIANT));
    }

    #[test]
    fn optimization_batch_fg_editor393_non_table_nodes_keep_variant_without_context_tier() {
        let node = host_contract::TemplatePaneNodeData {
            role: "Button".into(),
            component_role: "button".into(),
            component_variant: "outlined".into(),
            ..host_contract::TemplatePaneNodeData::default()
        };

        let node = apply_table_layout_context_variant(node, 640.0);

        assert_eq!(node.component_variant.as_str(), "outlined");
    }

    #[test]
    fn optimization_batch_fg_editor393_variant_append_preserves_trimmed_bytes() {
        for (variant, token, expected) in [
            ("", "layoutNarrow", "layoutNarrow"),
            (
                "  asset-table  ",
                "layoutNarrow",
                "asset-table layoutNarrow",
            ),
            (
                "asset-table layoutNarrow",
                "layoutNarrow",
                "asset-table layoutNarrow",
            ),
        ] {
            assert_eq!(append_component_variant_token(variant, token), expected);
        }

        let production = include_str!("template_layout_context.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        assert!(!production.contains("format!("));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fg_editor393_direct_variant_append_benchmark() {
        const VARIANT: &str = "asset-table dense";
        const TOKEN: &str = "layoutNarrow";
        for _ in 0..4 {
            black_box(measure_variants(
                |value, token| format!("{} {}", value.trim(), token),
                VARIANT,
                TOKEN,
            ));
            black_box(measure_variants(
                append_component_variant_token,
                VARIANT,
                TOKEN,
            ));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_variants(
                    |value, token| format!("{} {}", value.trim(), token),
                    VARIANT,
                    TOKEN,
                ));
                optimized_samples.push(measure_variants(
                    append_component_variant_token,
                    VARIANT,
                    TOKEN,
                ));
            } else {
                optimized_samples.push(measure_variants(
                    append_component_variant_token,
                    VARIANT,
                    TOKEN,
                ));
                legacy_samples.push(measure_variants(
                    |value, token| format!("{} {}", value.trim(), token),
                    VARIANT,
                    TOKEN,
                ));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn measure_variants(
        mut build: impl FnMut(&str, &str) -> String,
        variant: &str,
        token: &str,
    ) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..VARIANTS_PER_SAMPLE {
            let result = black_box(build(black_box(variant), black_box(token)));
            checksum = checksum.wrapping_add(result.len());
            black_box(result);
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
            "EDITOR393_DIRECT_VARIANT_APPEND_BENCH_V1 sample_pairs={SAMPLE_PAIRS} variants_per_sample={VARIANTS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=20",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(80) / 100,
            "direct variant append must reduce P95 by at least 20%"
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
