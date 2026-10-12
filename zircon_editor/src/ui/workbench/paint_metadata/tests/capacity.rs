use super::{
    append_visible_group_rows, reserve_visible_group_rows, AssetContentRect, AssetContentRowGroup,
};

#[test]
fn editor760_visible_group_rows_reserve_exact_bound() {
    let groups = vec![
        AssetContentRowGroup {
            top: 0.0,
            bottom: 10.0,
            node_rows: vec![1, 2],
        },
        AssetContentRowGroup {
            top: 10.0,
            bottom: 20.0,
            node_rows: vec![3, 4, 5],
        },
        AssetContentRowGroup {
            top: 20.0,
            bottom: 30.0,
            node_rows: vec![6],
        },
    ];
    let mut rows = Vec::with_capacity(2);
    rows.extend([90, 91]);
    reserve_visible_group_rows(&mut rows, &groups, 1, 3);
    assert!(rows.capacity() >= rows.len() + 4);

    let damage_clip = AssetContentRect {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 20.0,
    };
    let visible_count = append_visible_group_rows(
        &mut rows,
        &groups,
        Some(damage_clip),
        0.0,
        0.0,
        0.0,
        damage_clip,
    );
    assert_eq!(visible_count, 2);
    assert_eq!(rows, vec![90, 91, 1, 2, 3, 4, 5]);
}

#[test]
#[ignore = "run in the managed Release validation batch"]
fn editor760_visible_group_rows_capacity_bench_v1() {
    let values: Vec<usize> = (0..4096).collect();
    let legacy_growth_events = append_growth_events(&values, None);
    let optimized_growth_events = append_growth_events(&values, Some(values.len()));
    eprintln!(
        "EDITOR760_VISIBLE_GROUP_ROWS_CAPACITY_BENCH_V1 values={} legacy_growth_events={} optimized_growth_events={}",
        values.len(),
        legacy_growth_events,
        optimized_growth_events
    );
    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
}

fn append_growth_events(values: &[usize], initial_capacity: Option<usize>) -> usize {
    let mut rows = initial_capacity.map_or_else(Vec::new, Vec::with_capacity);
    let mut previous_capacity = rows.capacity();
    let mut growth_events = 0;
    for value in values {
        rows.push(*value);
        if rows.capacity() != previous_capacity {
            growth_events += 1;
            previous_capacity = rows.capacity();
        }
    }
    growth_events
}
