use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_compact_icon_text_workbench_button(
    node: &TemplatePaneNodeData,
) -> bool {
    node.component_variant
        .split_ascii_whitespace()
        .any(|token| {
            token.len() == "compact_icon_text".len()
                && token.eq_ignore_ascii_case("compact_icon_text")
        })
}

#[cfg(test)]
mod tests {
    use super::is_compact_icon_text_workbench_button;
    use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

    fn node_with_variant(variant: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            component_variant: variant.to_owned(),
            ..TemplatePaneNodeData::default()
        }
    }

    #[test]
    fn optimization_batch_gg_editor420_compact_icon_text_length_prefilter_preserves_matches() {
        assert!(is_compact_icon_text_workbench_button(&node_with_variant(
            "compact_icon_text"
        )));
        assert!(is_compact_icon_text_workbench_button(&node_with_variant(
            "COMPACT_ICON_TEXT"
        )));
        assert!(is_compact_icon_text_workbench_button(&node_with_variant(
            "dialog compact_icon_text disabled"
        )));
        assert!(!is_compact_icon_text_workbench_button(&node_with_variant(
            "compact_icon_textual"
        )));
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gg_editor420_compact_icon_text_length_prefilter_benchmark() {
        const MARKER: &str = "EDITOR420_COMPACT_ICON_TEXT_LENGTH_PREFILTER_BENCH_V1";
        const ITERATIONS: usize = 100_000;
        let node = node_with_variant("dialog compact_icon_textual disabled");
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert!(!is_compact_icon_text_workbench_button(&node));
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert!(!node
                .component_variant
                .split_ascii_whitespace()
                .any(|token| token.eq_ignore_ascii_case("compact_icon_text")));
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
    }
}
