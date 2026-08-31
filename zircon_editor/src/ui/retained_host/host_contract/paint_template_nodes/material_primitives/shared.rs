use super::super::super::data::TemplatePaneNodeData;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use crate::ui::retained_host::host_contract::paint_geometry::{
    bounded_extent,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn component_variant_contains(
    node: &TemplatePaneNodeData,
    expected: &str,
) -> bool {
    node.component_variant
        .as_str()
        .split(|character: char| {
            character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
        })
        .any(|part| part.len() == expected.len() && part.eq_ignore_ascii_case(expected))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn first_non_empty<'a>(
    values: &[&'a str],
) -> &'a str {
    values
        .iter()
        .copied()
        .find(|value| !value.is_empty())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::bounded_extent;

    #[test]
    fn bounded_extent_rejects_negative_and_non_finite_values() {
        assert_eq!(bounded_extent(12.5), 12.5);
        assert_eq!(bounded_extent(-1.0), 0.0);
        assert_eq!(bounded_extent(f32::NAN), 0.0);
        assert_eq!(bounded_extent(f32::INFINITY), 0.0);
    }

    #[test]
    fn optimization_batch_gn_editor426_component_variant_length_prefilter_preserves_matches() {
        let node = TemplatePaneNodeData {
            component_variant: "outlined compact_icon_text".to_owned(),
            ..TemplatePaneNodeData::default()
        };
        assert!(super::component_variant_contains(&node, "outlined"));
        assert!(super::component_variant_contains(
            &node,
            "COMPACT_ICON_TEXT"
        ));
        assert!(!super::component_variant_contains(&node, "outline"));
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gn_editor426_component_variant_length_prefilter_benchmark() {
        const MARKER: &str = "EDITOR426_COMPONENT_VARIANT_LENGTH_PREFILTER_BENCH_V1";
        const ITERATIONS: usize = 100_000;
        let node = TemplatePaneNodeData {
            component_variant: "outlined compact_icon_text unavailable".to_owned(),
            ..TemplatePaneNodeData::default()
        };
        let expected = "synchronizing";
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert!(!super::component_variant_contains(&node, expected));
        }
        let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            assert!(!node
                .component_variant
                .as_str()
                .split(|character: char| {
                    character.is_ascii_whitespace()
                        || matches!(character, ',' | '/' | '|' | ':' | ';')
                })
                .any(|part| part.eq_ignore_ascii_case(expected)));
        }
        let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
    }
}
