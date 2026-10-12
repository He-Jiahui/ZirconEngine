use std::collections::HashSet;

use super::admit_renderable_owner;

fn admitted_owners(owners: &[u64]) -> (Vec<u64>, Option<HashSet<u64>>) {
    let mut admitted = Vec::with_capacity(owners.len());
    let mut previous_owner = None;
    let mut seen_owners = None;
    for &owner in owners {
        if admit_renderable_owner(
            owner,
            &mut previous_owner,
            &mut seen_owners,
            admitted.iter().copied(),
        ) {
            admitted.push(owner);
        }
    }
    (admitted, seen_owners)
}

#[test]
fn grouped_owner_sequence_keeps_lazy_index_unallocated() {
    let (owners, seen_owners) = admitted_owners(&[1, 1, 2, 2, 3, 3]);

    assert_eq!(owners, [1, 2, 3]);
    assert!(seen_owners.is_none());
}

#[test]
fn interleaved_owner_sequence_collapses_non_adjacent_duplicates() {
    let (owners, seen_owners) = admitted_owners(&[1, 2, 3, 1, 2, 3]);

    assert_eq!(owners, [1, 2, 3]);
    assert!(seen_owners.is_some());
}
