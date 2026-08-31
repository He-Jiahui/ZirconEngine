use super::super::{
    theme_authoring::{
        build_imported_theme_local_merge_preview, build_theme_refactor_items,
        build_theme_rule_helper_items, can_prune_duplicate_local_theme_overrides,
    },
    theme_cascade_inspection::build_theme_cascade_inspection,
    theme_compare::build_theme_compare_items,
    theme_summary::{build_theme_source_details, build_theme_summary},
    ui_asset_editor_session::UiAssetEditorSession,
};
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(super) struct UiAssetThemePaneData {
    pub(super) source_items: Vec<String>,
    pub(super) source_selected_index: i32,
    pub(super) selected_source_reference: String,
    pub(super) selected_source_kind: String,
    pub(super) selected_source_token_count: i32,
    pub(super) selected_source_rule_count: i32,
    pub(super) selected_source_available: bool,
    pub(super) can_promote_local: bool,
    pub(super) selected_source_token_items: Vec<String>,
    pub(super) selected_source_rule_items: Vec<String>,
    pub(super) cascade_layer_items: Vec<String>,
    pub(super) cascade_token_items: Vec<String>,
    pub(super) cascade_rule_items: Vec<String>,
    pub(super) compare_items: Vec<String>,
    pub(super) merge_preview_items: Vec<String>,
    pub(super) rule_helper_items: Vec<String>,
    pub(super) refactor_items: Vec<String>,
    pub(super) promote_asset_id: String,
    pub(super) promote_document_id: String,
    pub(super) promote_display_name: String,
    pub(super) can_edit_promote_draft: bool,
    pub(super) can_prune_duplicate_local_overrides: bool,
}

impl UiAssetEditorSession {
    pub(super) fn theme_pane_presentation(&self) -> UiAssetThemePaneData {
        zircon_runtime::profile_scope!("editor", "asset_editor.presentation", "theme",);
        let summary = build_theme_summary(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        );
        let source_details = build_theme_source_details(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        );
        let cascade = build_theme_cascade_inspection(
            &self.last_valid_document,
            &self.compiler_imports.styles,
        );
        let compare_items = build_theme_compare_items(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        );
        let rule_helper_items = build_theme_rule_helper_items(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        );
        let refactor_items =
            build_theme_refactor_items(&self.last_valid_document, &self.compiler_imports.styles);
        let merge_preview_items = self
            .selected_theme_source_key
            .as_deref()
            .filter(|key| *key != "local")
            .and_then(|reference| {
                self.compiler_imports
                    .styles
                    .get(reference)
                    .map(|imported_style| {
                        build_imported_theme_local_merge_preview(
                            &self.last_valid_document,
                            reference,
                            imported_style,
                        )
                    })
            })
            .unwrap_or_default();
        let promote_draft = self.selected_promote_theme_draft();
        let can_edit_promote_draft = summary.selected_kind == "Local" && summary.can_promote_local;
        record_current_ui_perf_counter(UiPerfCounter::AssetEditorPaneThemeBuildCount, 1.0);
        UiAssetThemePaneData {
            source_items: summary.items,
            source_selected_index: summary.selected_index,
            selected_source_reference: summary.selected_reference,
            selected_source_kind: summary.selected_kind,
            selected_source_token_count: summary.selected_token_count,
            selected_source_rule_count: summary.selected_rule_count,
            selected_source_available: summary.selected_available,
            can_promote_local: summary.can_promote_local,
            selected_source_token_items: source_details.token_items,
            selected_source_rule_items: source_details.rule_items,
            cascade_layer_items: cascade.layer_items,
            cascade_token_items: cascade.token_items,
            cascade_rule_items: cascade.rule_items,
            compare_items,
            merge_preview_items,
            rule_helper_items,
            refactor_items,
            promote_asset_id: promote_draft
                .as_ref()
                .map(|draft| draft.asset_id.clone())
                .unwrap_or_default(),
            promote_document_id: promote_draft
                .as_ref()
                .map(|draft| draft.document_id.clone())
                .unwrap_or_default(),
            promote_display_name: promote_draft
                .as_ref()
                .map(|draft| draft.display_name.clone())
                .unwrap_or_default(),
            can_edit_promote_draft,
            can_prune_duplicate_local_overrides: can_prune_duplicate_local_theme_overrides(
                &self.last_valid_document,
                &self.compiler_imports.styles,
            ),
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    #[test]
    fn optimization_batch_ee_theme_kind_is_checked_before_direct_move() {
        let source = include_str!("theme.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("theme presentation production implementation");
        let check = production
            .find("let can_edit_promote_draft =")
            .expect("theme editability check");
        let direct_move = production
            .find("selected_source_kind: summary.selected_kind,")
            .expect("selected theme kind direct move");

        assert!(check < direct_move);
        assert!(!production.contains("selected_kind.clone()"));
    }

    #[test]
    #[ignore = "release-only direct theme-kind move benchmark"]
    fn optimization_batch_ee_direct_theme_kind_move_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const PROJECTIONS_PER_SAMPLE: usize = 16_384;

        fn measure_legacy() -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..PROJECTIONS_PER_SAMPLE {
                let selected_kind = black_box(String::from("Local"));
                let projected_kind = black_box(selected_kind.clone());
                let can_edit = black_box(selected_kind == "Local");
                checksum = checksum.wrapping_add(projected_kind.len() + usize::from(can_edit));
                black_box(projected_kind);
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized() -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..PROJECTIONS_PER_SAMPLE {
                let selected_kind = black_box(String::from("Local"));
                let can_edit = black_box(selected_kind == "Local");
                let projected_kind = black_box(selected_kind);
                checksum = checksum.wrapping_add(projected_kind.len() + usize::from(can_edit));
                black_box(projected_kind);
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy());
                optimized_samples.push(measure_optimized());
            } else {
                optimized_samples.push(measure_optimized());
                legacy_samples.push(measure_legacy());
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "EDITOR367_DIRECT_THEME_KIND_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             projections_per_sample={PROJECTIONS_PER_SAMPLE} pair_order=alternating_legacy_even \
             legacy_kind_clones_per_sample={PROJECTIONS_PER_SAMPLE} \
             optimized_kind_clones_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(75),
            "moving the theme kind must reduce P95 by at least 25%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
