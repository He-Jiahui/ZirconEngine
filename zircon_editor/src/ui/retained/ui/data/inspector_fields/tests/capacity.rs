const BENCHMARK_MARKER: &str = "EDITOR857_INSPECTOR_FIELD_NODE_CAPACITY_BENCH_V1";

#[test]
fn editor857_inspector_field_node_capacity_preserves_order() {
    let components = [(true, 2_usize), (false, 1), (true, 3)];
    let (nodes, growth_events) = model_node_count(components.iter().copied());

    assert_eq!(nodes, 4 + 2 + 5);
    assert_eq!(growth_events, 0);
}

#[test]
fn editor857_inspector_field_node_capacity_keeps_empty_projection_zero_growth() {
    let (nodes, growth_events) = model_node_count(std::iter::empty());

    assert_eq!(nodes, 0);
    assert_eq!(growth_events, 0);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor857_inspector_field_node_capacity_bench() {
    let components = (0..1_024).map(|index| (index % 2 == 0, index % 7));
    let (nodes, growth_events) = model_node_count(components);

    println!(
        "{BENCHMARK_MARKER} components=1024 projected_nodes={nodes} growth_events={growth_events}"
    );
    assert_eq!(growth_events, 0);
}

fn model_node_count<I>(components: I) -> (usize, usize)
where
    I: IntoIterator<Item = (bool, usize)>,
{
    let component_values = components.into_iter().collect::<Vec<_>>();
    let capacity = component_values
        .iter()
        .map(|(has_diagnostic, properties)| {
            1usize
                .saturating_add(usize::from(*has_diagnostic))
                .saturating_add(*properties)
        })
        .sum();
    let mut projected = Vec::with_capacity(capacity);
    let mut growth_events = 0;
    for (has_diagnostic, properties) in component_values {
        let count = 1usize
            .saturating_add(usize::from(has_diagnostic))
            .saturating_add(properties);
        for _ in 0..count {
            if projected.len() == projected.capacity() {
                growth_events += 1;
            }
            projected.push(());
        }
    }
    (projected.len(), growth_events)
}
