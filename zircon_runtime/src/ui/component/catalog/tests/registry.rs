use super::{unique_component_categories, UiComponentCategory};

#[test]
fn allocation_free_categories_preserve_enum_order() {
    let categories = unique_component_categories([
        UiComponentCategory::Feedback,
        UiComponentCategory::Numeric,
        UiComponentCategory::Visual,
    ])
    .collect::<Vec<_>>();

    assert_eq!(
        categories,
        [
            UiComponentCategory::Visual,
            UiComponentCategory::Numeric,
            UiComponentCategory::Feedback,
        ]
    );
}

#[test]
fn allocation_free_categories_deduplicate_repeated_values() {
    let categories = unique_component_categories([
        UiComponentCategory::Input,
        UiComponentCategory::Input,
        UiComponentCategory::Selection,
        UiComponentCategory::Input,
    ])
    .collect::<Vec<_>>();

    assert_eq!(
        categories,
        [UiComponentCategory::Input, UiComponentCategory::Selection]
    );
}

#[test]
fn allocation_free_categories_handle_empty_input() {
    assert_eq!(unique_component_categories([]).next(), None);
}

#[test]
fn optimization_batch_20260830cy_host_descriptors_reserve_registry_bound() {
    let source = include_str!("../registry.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("component registry production source");

    assert!(production.contains("Vec::with_capacity(self.descriptors.len())"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830cy_host_descriptor_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const DESCRIPTOR_COUNT: usize = 32;
    const MATCHED_DESCRIPTOR_COUNT: usize = 24;
    const MARKER: &str = "RUNTIME511_HOST_DESCRIPTOR_CAPACITY_BENCH_V1";

    let legacy_growth_events = descriptor_growth_events(
        BATCH_COUNT,
        DESCRIPTOR_COUNT,
        MATCHED_DESCRIPTOR_COUNT,
        false,
    );
    let optimized_growth_events = descriptor_growth_events(
        BATCH_COUNT,
        DESCRIPTOR_COUNT,
        MATCHED_DESCRIPTOR_COUNT,
        true,
    );

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} descriptor_count={DESCRIPTOR_COUNT} \
             matched_descriptor_count={MATCHED_DESCRIPTOR_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn descriptor_growth_events(
    batch_count: usize,
    descriptor_count: usize,
    matched_descriptor_count: usize,
    reserve: bool,
) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let mut descriptors = if reserve {
            Vec::with_capacity(descriptor_count)
        } else {
            Vec::new()
        };
        for descriptor in 0..matched_descriptor_count {
            let previous_capacity = descriptors.capacity();
            descriptors.push(descriptor);
            growth_events += usize::from(descriptors.capacity() != previous_capacity);
        }
    }
    growth_events
}
