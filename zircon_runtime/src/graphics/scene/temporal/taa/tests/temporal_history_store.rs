use super::TemporalHistoryState;

#[test]
fn temporal_history_state_starts_invalid_and_flips_read_write_slots() {
    let mut state = TemporalHistoryState::default();

    assert_eq!(state.read_index, 0);
    assert_eq!(state.write_index(), 1);
    state.flip_after_success();

    assert_eq!(state.read_index, 1);
    assert_eq!(state.write_index(), 0);
}
