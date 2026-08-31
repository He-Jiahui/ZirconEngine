use std::collections::BTreeSet;

use crate::ui::asset_editor::{UiAssetEditorRoute, UiAssetEditorSession};
use crate::ui::host::editor_error::EditorError;
use crate::ui::host::project_access::normalize_ui_asset_asset_id;

use super::super::{build_ui_asset_editor_session_from_source, preview_size_for_preset};

pub(in crate::ui::host::asset_editor_sessions) fn normalize_ui_asset_change_set<I, S>(
    changed_asset_ids: I,
) -> BTreeSet<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut normalized = BTreeSet::new();
    for asset_id in changed_asset_ids {
        let asset_id = normalize_ui_asset_asset_id(asset_id.as_ref());
        if !normalized.contains(asset_id) {
            normalized.insert(asset_id.to_owned());
        }
    }
    normalized
}

pub(super) fn rebuild_ui_asset_session_from_source(
    route: UiAssetEditorRoute,
    source: String,
) -> Result<UiAssetEditorSession, EditorError> {
    let preview_size = preview_size_for_preset(route.preview_preset);
    build_ui_asset_editor_session_from_source(route, source, preview_size)
        .map_err(|error| EditorError::UiAsset(error.to_string()))
}

#[cfg(test)]
mod performance_tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::Instant;

    use super::normalize_ui_asset_change_set;
    use crate::ui::host::project_access::normalize_ui_asset_asset_id;

    #[test]
    fn changed_import_lookup_borrows_the_normalized_asset_id() {
        let source = include_str!("imports.rs");
        let owned_lookup = [
            "contains(&normalize_ui_asset_asset_id(reference)",
            ".to_string())",
        ]
        .concat();

        assert!(!source.contains(&owned_lookup));
    }

    #[test]
    fn optimization_batch_dw_asset_change_dedup_preserves_normalized_unique_ids() {
        let changed = [
            "res://ui/shared.widget#header",
            "res://ui/shared.widget#footer",
            "res://ui/theme.style",
            "res://ui/theme.style#dark",
        ];

        let normalized = normalize_ui_asset_change_set(changed);

        assert_eq!(
            normalized,
            BTreeSet::from([
                "res://ui/shared.widget".to_owned(),
                "res://ui/theme.style".to_owned(),
            ])
        );
    }

    #[test]
    fn optimization_batch_dw_asset_change_dedup_checks_before_allocating() {
        let production = include_str!("normalize.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("asset change normalization production source");

        assert!(production.contains("if !normalized.contains(asset_id)"));
        assert!(production.contains("normalized.insert(asset_id.to_owned())"));
        assert!(!production.contains(".map(|asset_id|"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dw_borrowed_asset_change_dedup_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const NORMALIZATIONS_PER_SAMPLE: usize = 256;
        const EVENT_COUNT: usize = 2_048;
        const UNIQUE_ASSET_COUNT: usize = 32;

        let changed_asset_ids = (0..EVENT_COUNT)
            .map(|index| {
                format!(
                    "res://ui/{}/asset_{:04}.widget#node_{index:04}",
                    "long_asset_segment/".repeat(8),
                    index % UNIQUE_ASSET_COUNT
                )
            })
            .collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_normalization(
                    &changed_asset_ids,
                    NORMALIZATIONS_PER_SAMPLE,
                    false,
                ));
                optimized_samples.push(measure_normalization(
                    &changed_asset_ids,
                    NORMALIZATIONS_PER_SAMPLE,
                    true,
                ));
            } else {
                optimized_samples.push(measure_normalization(
                    &changed_asset_ids,
                    NORMALIZATIONS_PER_SAMPLE,
                    true,
                ));
                legacy_samples.push(measure_normalization(
                    &changed_asset_ids,
                    NORMALIZATIONS_PER_SAMPLE,
                    false,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "EDITOR359_BORROWED_UI_ASSET_CHANGE_DEDUP_BENCH_V1 normalizations_per_sample={NORMALIZATIONS_PER_SAMPLE} event_count={EVENT_COUNT} unique_asset_count={UNIQUE_ASSET_COUNT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "borrowed asset change dedup p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn measure_normalization(
        changed_asset_ids: &[String],
        normalization_count: usize,
        optimized: bool,
    ) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..normalization_count {
            let normalized = if optimized {
                normalize_ui_asset_change_set(changed_asset_ids.iter())
            } else {
                changed_asset_ids
                    .iter()
                    .map(|asset_id| normalize_ui_asset_asset_id(asset_id).to_string())
                    .collect::<BTreeSet<_>>()
            };
            checksum = checksum.wrapping_add(normalized.len());
            black_box(normalized);
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
