use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn variant_contains_any(
    node: &TemplatePaneNodeData,
    expected: &[&str],
) -> bool {
    [
        node.component_variant.as_str(),
        node.surface_variant.as_str(),
        node.validation_level.as_str(),
        node.text_tone.as_str(),
        node.button_variant.as_str(),
    ]
    .iter()
    .flat_map(|value| value.split_whitespace())
    .any(|part| {
        expected
            .iter()
            .any(|expected| part.len() == expected.len() && part.eq_ignore_ascii_case(expected))
    })
}

#[cfg(test)]
mod tests {
    use super::variant_contains_any;
    use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

    fn node_with_component_variant(component_variant: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            component_variant: component_variant.to_owned(),
            ..TemplatePaneNodeData::default()
        }
    }

    #[test]
    fn optimization_batch_gf_editor418_variant_length_prefilter_preserves_matches() {
        let node = node_with_component_variant("Dialog disabled");
        assert!(variant_contains_any(&node, &["dialog"]));
        assert!(variant_contains_any(&node, &["DISABLED"]));
        assert!(!variant_contains_any(&node, &["dialogue", "enable"]));
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gf_editor418_variant_length_prefilter_benchmark() {
        const MARKER: &str = "EDITOR418_VARIANT_LENGTH_PREFILTER_BENCH_V1";
        const ITERATIONS: usize = 100_000;
        let node = node_with_component_variant("dialog disabled");
        let expected = ["confirmation", "cancelled", "synchronizing", "unavailable"];
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert!(!variant_contains_any(&node, &expected));
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert!(![
                node.component_variant.as_str(),
                node.surface_variant.as_str(),
                node.validation_level.as_str(),
                node.text_tone.as_str(),
                node.button_variant.as_str(),
            ]
            .iter()
            .flat_map(|value| value.split_whitespace())
            .any(|part| expected
                .iter()
                .any(|expected| part.eq_ignore_ascii_case(expected))));
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
    }
}
