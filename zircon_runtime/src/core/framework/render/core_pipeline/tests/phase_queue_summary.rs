#[test]
fn summary_updates_phase_and_order_rows_without_per_item_linear_search() {
    let source = include_str!("../phase_queue_summary.rs");

    assert!(source.contains(concat!("phase_count_index", "(item.phase)")));
    assert!(source.contains(concat!("usize::from(item.phase.", "queue_order())")));
    assert!(!source.contains(concat!(".find(|count| count.phase", " == item.phase)")));
    assert!(!source.contains(concat!(
        ".find(|span| span.phase_order",
        " == item.phase.queue_order())"
    )));
    assert!(!source.contains(concat!(".collect::<Vec<_>>()", ".join(\"+\")")));
    assert!(source.contains(concat!("String::with_", "capacity(capacity)")));
}
