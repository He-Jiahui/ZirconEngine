use super::ConfigPersistenceState;

#[test]
fn unchanged_value_does_not_postpone_an_already_requested_generation() {
    let mut state = ConfigPersistenceState::default();
    assert!(state.request_persistence(true));
    let first_dirty_at = state.last_dirty_at;

    assert!(!state.request_persistence(false));
    assert_eq!(state.last_dirty_at, first_dirty_at);
    assert_eq!(state.dirty_generation, 1);
}
