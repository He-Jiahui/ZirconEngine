use std::sync::atomic::{AtomicU64, Ordering};

use super::advance_sequence;

#[test]
fn final_catalog_input_sequence_is_published_once_without_wrapping() {
    let sequence = AtomicU64::new(u64::MAX - 1);

    assert_eq!(advance_sequence(&sequence), u64::MAX - 1);
    assert_eq!(sequence.load(Ordering::Relaxed), u64::MAX);
}

#[test]
#[should_panic(expected = "project catalog input generation sequence exhausted")]
fn exhausted_catalog_input_sequence_never_reuses_an_old_generation() {
    let sequence = AtomicU64::new(u64::MAX);

    let _ = advance_sequence(&sequence);
}
