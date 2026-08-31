use toml::Value;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiChildMount, UiNodeDefinition, UiStyleDeclarationBlock,
};

pub(super) fn visit_resource_uris<'a>(
    document: &'a UiAssetDocument,
    mut visit: impl FnMut(&'a str),
) {
    for reference in &document.imports.resources {
        visit_uri(&reference.uri, &mut visit);
        if let Some(uri) = reference.fallback.uri.as_deref() {
            visit_uri(uri, &mut visit);
        }
    }
    for value in document.tokens.values() {
        visit_value(value, &mut visit);
    }
    if let Some(root) = &document.root {
        visit_node(root, &mut visit);
    }
    for component in document.components.values() {
        visit_node(&component.root, &mut visit);
    }
    for stylesheet in &document.stylesheets {
        for rule in &stylesheet.rules {
            visit_declaration_block(&rule.set, &mut visit);
        }
    }
}

fn visit_node<'a>(node: &'a UiNodeDefinition, visit: &mut impl FnMut(&'a str)) {
    visit_values(node.props.values(), visit);
    visit_values(node.params.values(), visit);
    if let Some(layout) = &node.layout {
        visit_values(layout.values(), visit);
    }
    visit_declaration_block(&node.style_overrides, visit);
    for child in &node.children {
        visit_child(child, visit);
    }
}

fn visit_child<'a>(child: &'a UiChildMount, visit: &mut impl FnMut(&'a str)) {
    visit_values(child.slot.values(), visit);
    visit_node(&child.node, visit);
}

fn visit_declaration_block<'a>(
    block: &'a UiStyleDeclarationBlock,
    visit: &mut impl FnMut(&'a str),
) {
    visit_values(block.self_values.values(), visit);
    visit_values(block.slot.values(), visit);
}

fn visit_values<'a>(values: impl Iterator<Item = &'a Value>, visit: &mut impl FnMut(&'a str)) {
    for value in values {
        visit_value(value, visit);
    }
}

fn visit_value<'a>(value: &'a Value, visit: &mut impl FnMut(&'a str)) {
    match value {
        Value::String(uri) => visit_uri(uri, visit),
        Value::Array(values) => visit_values(values.iter(), visit),
        Value::Table(table) => visit_values(table.values(), visit),
        _ => {}
    }
}

fn visit_uri<'a>(uri: &'a str, visit: &mut impl FnMut(&'a str)) {
    if uri.starts_with("res://") || uri.starts_with("asset://") || uri.starts_with("project://") {
        visit(uri);
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use toml::Value;

    use super::visit_value;

    const SAMPLE_PAIRS: usize = 17;
    const RESOURCE_URI_COUNT: usize = 4_096;

    fn legacy_collect_value<'a>(value: &'a Value, uris: &mut Vec<&'a str>) {
        match value {
            Value::String(uri) => {
                if uri.starts_with("res://")
                    || uri.starts_with("asset://")
                    || uri.starts_with("project://")
                {
                    uris.push(uri);
                }
            }
            Value::Array(values) => {
                for value in values {
                    legacy_collect_value(value, uris);
                }
            }
            Value::Table(table) => {
                for value in table.values() {
                    legacy_collect_value(value, uris);
                }
            }
            _ => {}
        }
    }

    fn fixture() -> Value {
        Value::Array(
            (0..RESOURCE_URI_COUNT)
                .map(|index| Value::String(format!("res://textures/ui/resource_{index:04}.png")))
                .collect(),
        )
    }

    fn elapsed_nanos(run: impl FnOnce()) -> u128 {
        let started = Instant::now();
        run();
        started.elapsed().as_nanos().max(1)
    }

    fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
        samples.sort_unstable();
        let rank = (samples.len() * percentile).div_ceil(100);
        samples[rank.saturating_sub(1)]
    }

    #[test]
    fn runtime74_batch_ui_resource_reference_visitor_preserves_order() {
        let value = Value::Array(vec![
            Value::String("res://textures/a.png".to_string()),
            Value::String("plain text".to_string()),
            Value::Table(toml::Table::from_iter([
                (
                    "asset".to_string(),
                    Value::String("asset://fonts/b.font".to_string()),
                ),
                (
                    "project".to_string(),
                    Value::String("project://theme/c.toml".to_string()),
                ),
            ])),
        ]);
        let mut legacy = Vec::new();
        legacy_collect_value(&value, &mut legacy);
        let mut visited = Vec::new();
        visit_value(&value, &mut |uri| visited.push(uri));

        assert_eq!(visited, legacy);
        assert_eq!(
            visited,
            [
                "res://textures/a.png",
                "asset://fonts/b.font",
                "project://theme/c.toml"
            ]
        );
    }

    #[test]
    fn runtime74_batch_ui_resource_reference_visitor_has_no_temporary_uri_vector() {
        let source = include_str!("resource_references.rs")
            .split_once("#[cfg(test)]")
            .unwrap()
            .0;

        assert!(source.contains("pub(super) fn visit_resource_uris"));
        assert!(source.contains("visit: &mut impl FnMut(&'a str)"));
        assert!(!source.contains("Vec<&str>"));
    }

    #[test]
    #[ignore = "release performance evidence for the managed validation coordinator"]
    fn runtime74_batch_ui_resource_reference_visitor_performance_evidence() {
        let value = fixture();

        for _ in 0..4 {
            let mut legacy = Vec::new();
            legacy_collect_value(black_box(&value), &mut legacy);
            black_box(legacy);
            let mut visited = 0usize;
            visit_value(black_box(&value), &mut |uri| {
                black_box(uri);
                visited = visited.saturating_add(1);
            });
            assert_eq!(visited, RESOURCE_URI_COUNT);
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            let measure_legacy = || {
                elapsed_nanos(|| {
                    let mut uris = Vec::new();
                    legacy_collect_value(black_box(&value), &mut uris);
                    for uri in &uris {
                        black_box(*uri);
                    }
                    assert_eq!(black_box(uris.len()), RESOURCE_URI_COUNT);
                })
            };
            let measure_optimized = || {
                elapsed_nanos(|| {
                    let mut visited = 0usize;
                    visit_value(black_box(&value), &mut |uri| {
                        black_box(uri);
                        visited = visited.saturating_add(1);
                    });
                    assert_eq!(black_box(visited), RESOURCE_URI_COUNT);
                })
            };
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_legacy());
                optimized_samples.push(measure_optimized());
            } else {
                optimized_samples.push(measure_optimized());
                legacy_samples.push(measure_legacy());
            }
        }

        let legacy_p50 = nearest_rank(&mut legacy_samples.clone(), 50);
        let legacy_p95 = nearest_rank(&mut legacy_samples, 95);
        let optimized_p50 = nearest_rank(&mut optimized_samples.clone(), 50);
        let optimized_p95 = nearest_rank(&mut optimized_samples, 95);
        println!(
            "RUNTIME74_UI_RESOURCE_REFERENCE_VISITOR_BENCH_V1 sample_pairs={} resource_uris={} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_temporary_uri_vectors=1 optimized_temporary_uri_vectors=0 uri_string_clones=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
            SAMPLE_PAIRS,
            RESOURCE_URI_COUNT,
            legacy_p50,
            legacy_p95,
            optimized_p50,
            optimized_p95,
            legacy_samples,
            optimized_samples,
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(80),
            "streaming URI visitor p95 must be at least 20% below the temporary-vector path: legacy={legacy_p95}us optimized={optimized_p95}us"
        );
    }
}
