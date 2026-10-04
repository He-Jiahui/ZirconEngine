use super::next_start_index;

#[test]
fn global_budget_starts_next_pump_at_first_unvisited_consumer() {
    assert_eq!(next_start_index(4, 3), Some(3));
    assert_eq!(next_start_index(4, 4), Some(0));
}

#[test]
fn empty_consumer_snapshot_has_no_next_start_index() {
    assert_eq!(next_start_index(0, 0), None);
    assert_eq!(next_start_index(0, 1), None);
    assert_eq!(next_start_index(0, usize::MAX), None);
    assert_eq!(next_start_index(3, 0), None);
}
