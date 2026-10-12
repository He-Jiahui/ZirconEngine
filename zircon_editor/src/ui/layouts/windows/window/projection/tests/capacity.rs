use super::{page_tab_visible_capacity, visible_page_tab_indices};

#[test]
fn editor762_page_tab_visible_capacity_preserves_bounded_active_slot() {
    assert_eq!(page_tab_visible_capacity(2, 8, 300.0), 2);
    assert_eq!(page_tab_visible_capacity(8, 3, 300.0), 3);
    assert_eq!(page_tab_visible_capacity(0, 1, 0.0), 1);
}

#[test]
fn editor762_page_tab_visible_capacity_is_tight_for_a_finite_tab_lane() {
    let capacity = page_tab_visible_capacity(100_000, 100_000, 360.0);
    assert!(capacity < 100_000);
    assert!(capacity >= 1);
}

#[test]
fn editor762_page_tab_projection_still_returns_an_ordered_visible_prefix() {
    let tabs = super::model_rc(vec![
        super::test_tab("First", true, false),
        super::test_tab("Second", false, false),
    ]);
    let visible = visible_page_tab_indices(&tabs, 1280.0);
    assert!(!visible.is_empty());
    assert_eq!(visible[0], 0);
    assert!(visible.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
#[ignore = "run in the managed Release validation batch"]
fn editor762_page_tab_capacity_bench_v1() {
    let values: Vec<usize> = (0..4096).collect();
    let legacy_growth_events = append_growth_events(&values, None);
    let optimized_growth_events = append_growth_events(&values, Some(values.len()));
    eprintln!(
        "EDITOR762_PAGE_TAB_CAPACITY_BENCH_V1 values={} legacy_growth_events={} optimized_growth_events={}",
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
