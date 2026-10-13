use super::tests::apply_same_topology_change;

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor815_play_hierarchy_changed_row_capacity_bench_v1() {
    const ROW_COUNT: usize = 4096;
    let (dense_projection, dense_fragment) = apply_same_topology_change(ROW_COUNT, ROW_COUNT);
    let dense_fragment = dense_fragment.expect("dense changes must publish a patch");
    let dense_capacity = dense_projection.changed_row_capacity.as_ref().unwrap();
    let optimized_growth_events = dense_capacity.growth_events;
    let mut legacy_changed_rows = Vec::new();
    let mut legacy_growth_events = 0;
    for row in dense_fragment.changed_rows().unwrap() {
        let previous_capacity = legacy_changed_rows.capacity();
        legacy_changed_rows.push(row.clone());
        if legacy_changed_rows.capacity() != previous_capacity {
            legacy_growth_events += 1;
        }
    }
    assert_eq!(legacy_changed_rows.len(), ROW_COUNT);
    assert_eq!(
        legacy_changed_rows.as_slice(),
        dense_fragment.changed_rows().unwrap()
    );
    assert_eq!(dense_capacity.initial_capacity, ROW_COUNT);
    assert_eq!(dense_capacity.final_capacity, ROW_COUNT);

    let (zero_projection, zero_fragment) = apply_same_topology_change(ROW_COUNT, 0);
    assert!(zero_fragment.is_none());
    let zero_capacity = zero_projection.changed_row_capacity.as_ref().unwrap();
    assert_eq!(zero_capacity.initial_capacity, 0);
    assert_eq!(zero_capacity.final_capacity, 0);
    assert_eq!(zero_capacity.growth_events, 0);
    let (sparse_projection, sparse_fragment) = apply_same_topology_change(ROW_COUNT, 1);
    assert_eq!(sparse_fragment.unwrap().changed_rows().unwrap().len(), 1);
    let sparse_capacity = sparse_projection.changed_row_capacity.as_ref().unwrap();
    assert_eq!(sparse_capacity.initial_capacity, 1);
    assert_eq!(sparse_capacity.final_capacity, 1);
    assert_eq!(sparse_capacity.growth_events, 0);

    println!(
        "EDITOR815_PLAY_HIERARCHY_CHANGED_ROW_CAPACITY_BENCH_V1 rows={ROW_COUNT} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} zero_capacity={} sparse_capacity={} dense_capacity={} producer=PlayHierarchyProjection::apply",
        zero_capacity.initial_capacity,
        sparse_capacity.initial_capacity,
        dense_capacity.initial_capacity,
    );
    assert_eq!(legacy_growth_events, 11);
    assert_eq!(optimized_growth_events, 0);
    assert!(legacy_growth_events > optimized_growth_events);
}
