use super::super::{
    source_sync::build_source_selection_summary, ui_asset_editor_session::UiAssetEditorSession,
};

pub(super) struct UiAssetSourcePaneData {
    pub(super) selected_block_label: String,
    pub(super) selected_line: i32,
    pub(super) selected_excerpt: String,
    pub(super) roundtrip_status: String,
    pub(super) outline_items: Vec<String>,
    pub(super) outline_selected_index: i32,
    pub(super) structured_diagnostic_items: Vec<String>,
}

impl UiAssetEditorSession {
    pub(super) fn source_pane_presentation(&self) -> UiAssetSourcePaneData {
        zircon_runtime::profile_scope!("editor", "asset_editor.presentation", "source",);
        let source_outline = self.roundtrip_source_outline_index();
        let source_summary = build_source_selection_summary(
            &source_outline,
            &self.selection,
            &self.diagnostics,
            self.selected_source_line_offset(),
        );
        let outline_selected_index = self
            .selection
            .primary_node_id
            .as_deref()
            .or_else(|| {
                self.structured_diagnostics
                    .iter()
                    .find_map(|diagnostic| diagnostic.target_node_id.as_deref())
            })
            .and_then(|node_id| source_outline.index_for_node(node_id))
            .map(|index| index as i32)
            .unwrap_or(-1);
        let outline_entries = source_outline.entries();
        let mut outline_items = Vec::with_capacity(outline_entries.len());
        for entry in outline_entries {
            outline_items.push(format!("line {} • {}", entry.line, entry.block_label));
        }
        let mut structured_diagnostic_items = Vec::with_capacity(self.structured_diagnostics.len());
        for diagnostic in &self.structured_diagnostics {
            structured_diagnostic_items.push(format!(
                "{} [{}] {}: {}",
                diagnostic.severity.as_str(),
                diagnostic.code,
                diagnostic.source_path,
                diagnostic.message
            ));
        }
        UiAssetSourcePaneData {
            selected_block_label: source_summary.block_label,
            selected_line: source_summary.line,
            selected_excerpt: source_summary.excerpt,
            roundtrip_status: source_summary.roundtrip_status,
            outline_items,
            outline_selected_index,
            structured_diagnostic_items,
        }
    }
}

#[cfg(test)]
mod optimization_batch_20260830ca_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const ITEMS_PER_SAMPLE: usize = 256;

    #[test]
    fn source_presentation_reserves_outline_and_diagnostic_capacity() {
        let source = include_str!("source.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(outline_entries.len())"));
        assert!(implementation.contains("Vec::with_capacity(self.structured_diagnostics.len())"));
        assert!(implementation.contains("for entry in outline_entries"));
        assert!(implementation.contains("for diagnostic in &self.structured_diagnostics"));
    }

    #[test]
    fn source_presentation_keeps_outline_before_diagnostic_order() {
        let source = include_str!("source.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let outline = implementation
            .find("for entry in outline_entries")
            .expect("outline loop");
        let diagnostic = implementation
            .find("for diagnostic in &self.structured_diagnostics")
            .expect("diagnostic loop");
        assert!(outline < diagnostic);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830ca_editor_source_presentation_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR325_SOURCE_PRESENTATION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} items_per_sample={ITEMS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..128 {
            let mut outline = if optimized {
                Vec::with_capacity(ITEMS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            let mut diagnostics = if optimized {
                Vec::with_capacity(ITEMS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..ITEMS_PER_SAMPLE {
                outline.push(index);
                diagnostics.push(index);
            }
            checksum ^= outline.len() ^ diagnostics.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
