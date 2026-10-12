use std::hint::black_box;

use super::*;

const STALE_ID_CAPACITY_MARKER: &str = "EDITOR874_NATIVE_WINDOW_STALE_ID_CAPACITY_BENCH_V1";

fn windows(count: usize) -> BTreeMap<MainPageId, ()> {
    (0..count)
        .map(|index| (MainPageId::new(format!("window:{index:04}")), ()))
        .collect()
}

#[test]
fn editor874_stale_native_window_ids_preserve_order_and_empty_capacity() {
    let windows = windows(6);
    let retained = BTreeSet::from([
        MainPageId::new("window:0001"),
        MainPageId::new("window:0003"),
        MainPageId::new("window:0005"),
    ]);
    let stale = stale_native_window_ids(&windows, &retained);
    assert_eq!(
        stale,
        vec![
            MainPageId::new("window:0000"),
            MainPageId::new("window:0002"),
            MainPageId::new("window:0004"),
        ]
    );
    assert!(stale.capacity() >= windows.len());

    let all = windows.keys().cloned().collect::<BTreeSet<_>>();
    let no_stale = stale_native_window_ids(&windows, &all);
    assert!(no_stale.is_empty());
    assert_eq!(no_stale.capacity(), 0);
}

#[test]
#[ignore = "release-only allocation-capacity performance evidence"]
fn editor874_native_window_stale_id_capacity_bench() {
    const WINDOW_COUNT: usize = 128;
    const QUERY_COUNT: usize = 65_536;
    let windows = windows(WINDOW_COUNT);
    let retained = windows
        .keys()
        .enumerate()
        .filter_map(|(index, window_id)| (index % 2 == 0).then(|| window_id.clone()))
        .collect::<BTreeSet<_>>();

    for _ in 0..QUERY_COUNT {
        black_box(stale_native_window_ids(
            black_box(&windows),
            black_box(&retained),
        ));
    }
    println!(
        "{STALE_ID_CAPACITY_MARKER} queries={QUERY_COUNT} windows={WINDOW_COUNT} \
             stale_per_query={} legacy_modeled_growth_events_per_query=5 \
             optimized_modeled_growth_events_per_query=0",
        WINDOW_COUNT / 2,
    );
}
