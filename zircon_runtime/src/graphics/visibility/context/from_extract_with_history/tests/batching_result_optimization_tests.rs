use super::*;

#[test]
fn optimization_batch_20260826n_runtime09b_hash_entity_sets_preserve_sorted_output() {
    let mut entities = EntitySet::with_capacity(5);
    for entity in [41, 7, 19, 7, 2] {
        entities.insert(entity);
    }

    assert_eq!(sorted_entity_ids(entities), vec![2, 7, 19, 41]);
}

#[test]
fn entity_set_keeps_monotonic_input_without_reordering() {
    let mut entities = EntitySet::with_capacity(4);
    for entity in [2, 7, 19, 41] {
        entities.insert(entity);
    }

    assert!(entities.input_was_sorted);
    assert_eq!(sorted_entity_ids(entities), vec![2, 7, 19, 41]);
}

#[test]
fn entity_set_reverses_monotonic_descending_input() {
    let mut entities = EntitySet::with_capacity(4);
    for entity in [41, 19, 7, 2] {
        entities.insert(entity);
    }

    assert!(entities.input_was_reverse_sorted);
    assert_eq!(sorted_entity_ids(entities), vec![2, 7, 19, 41]);
}

#[test]
fn entity_set_materializes_membership_only_after_mixed_input() {
    let mut entities = EntitySet::with_capacity(4);
    for entity in [2, 7, 2, 9, 1] {
        entities.insert(entity);
    }

    assert!(entities.membership.is_some());
    assert_eq!(sorted_entity_ids(entities), vec![1, 2, 7, 9]);
}
