use std::collections::BTreeMap;
use std::mem::size_of;
use std::rc::Rc;

use super::{PersistentRowPatchCursor, PersistentRowPatchMap};

#[test]
fn ordered_cursor_frontier_stays_within_one_pointer_per_trie_level() {
    let pointer_budget = (usize::BITS as usize + 8) * size_of::<usize>();
    assert!(
        size_of::<PersistentRowPatchCursor<'_, usize>>() <= pointer_budget,
        "cursor must not store row/depth metadata beside every pending node"
    );
}

#[test]
fn repeated_single_row_updates_keep_history_out_of_the_update_cost() {
    let mut patches = PersistentRowPatchMap::empty(10_000);
    let first_version = patches.with_updates(BTreeMap::from([(0, Rc::new(1usize))]));
    patches = first_version.clone();
    for row in 1..10_000 {
        patches = patches.with_updates(BTreeMap::from([(row, Rc::new(row + 1))]));
    }

    assert_eq!(patches.depth, 14);
    assert_eq!(patches.len(), 10_000);
    assert_eq!(patches.get(0).map(|value| **value), Some(1));
    assert_eq!(patches.get(9_999).map(|value| **value), Some(10_000));
    assert!(first_version.get(9_999).is_none());
}

#[test]
fn replacing_a_row_keeps_the_patch_cardinality_stable() {
    let patches =
        PersistentRowPatchMap::empty(16).with_updates(BTreeMap::from([(3, Rc::new("old"))]));
    let replaced = patches.with_updates(BTreeMap::from([(3, Rc::new("new"))]));

    assert_eq!(replaced.len(), 1);
    assert_eq!(replaced.get(3).map(|value| **value), Some("new"));
    assert_eq!(patches.get(3).map(|value| **value), Some("old"));
}

#[test]
fn ordered_cursor_visits_sparse_trie_nodes_instead_of_every_model_row() {
    let patches = PersistentRowPatchMap::empty(10_000).with_updates(BTreeMap::from([
        (3, Rc::new(30usize)),
        (5_000, Rc::new(50_000usize)),
        (9_999, Rc::new(99_990usize)),
    ]));
    let mut cursor = patches.forward_cursor();
    let resolved = (0..10_000)
        .filter_map(|row| cursor.value_at(row).map(|value| (row, **value)))
        .collect::<Vec<_>>();

    assert_eq!(resolved, vec![(3, 30), (5_000, 50_000), (9_999, 99_990)]);
    assert!(
        cursor.node_visits() < 128,
        "three sparse patches must not trigger one trie descent per model row"
    );
}

#[test]
fn reverse_cursor_resolves_patches_in_descending_row_order() {
    let patches = PersistentRowPatchMap::empty(32).with_updates(BTreeMap::from([
        (1, Rc::new(10usize)),
        (17, Rc::new(170usize)),
        (31, Rc::new(310usize)),
    ]));
    let mut cursor = patches.reverse_cursor();
    let resolved = (0..32)
        .rev()
        .filter_map(|row| cursor.value_at(row).map(|value| (row, **value)))
        .collect::<Vec<_>>();

    assert_eq!(resolved, vec![(31, 310), (17, 170), (1, 10)]);
    assert!(cursor.node_visits() < 64);
}

#[test]
fn cursor_reconstructs_the_highest_usize_row_bit() {
    if usize::BITS < 64 {
        return;
    }
    let high_row = 1usize << (usize::BITS - 1);
    let patches = PersistentRowPatchMap::empty(high_row + 1).with_updates(BTreeMap::from([
        (0, Rc::new(10usize)),
        (high_row, Rc::new(20usize)),
    ]));

    let mut forward = patches.forward_cursor();
    assert_eq!(forward.value_at(0).map(|value| **value), Some(10));
    assert_eq!(forward.value_at(high_row).map(|value| **value), Some(20));

    let mut reverse = patches.reverse_cursor();
    assert_eq!(reverse.value_at(high_row).map(|value| **value), Some(20));
    assert_eq!(reverse.value_at(0).map(|value| **value), Some(10));
}
