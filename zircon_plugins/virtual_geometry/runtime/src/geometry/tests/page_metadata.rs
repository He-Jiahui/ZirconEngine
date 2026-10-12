use std::collections::BTreeMap;

use super::*;

#[test]
fn replacing_parent_pages_rebuilds_sorted_child_index_and_descendants() {
    let mut state = VirtualGeometryRuntimeState::default();
    state.replace_page_parent_pages(BTreeMap::from([(30, 10), (20, 10), (40, 20)]));

    assert_eq!(state.page_dependency_count(), 3);
    assert_eq!(state.page_child_pages().get(&10), Some(&vec![20, 30]));
    assert_eq!(state.page_child_pages().get(&20), Some(&vec![40]));
    assert_eq!(state.page_descendant_ids(10), vec![20, 30, 40]);
}

#[test]
fn retaining_parent_pages_rebuilds_child_index_without_stale_children() {
    let mut state = VirtualGeometryRuntimeState::default();
    state.replace_page_parent_pages(BTreeMap::from([(20, 10), (30, 10), (40, 20)]));

    state.retain_page_parent_pages(|page_id, _| *page_id != 20);

    assert_eq!(state.page_dependency_count(), 1);
    assert_eq!(state.page_child_pages().get(&10), Some(&vec![30]));
    assert_eq!(state.page_child_pages().get(&20), None);
    assert_eq!(state.page_descendant_ids(10), vec![30]);
}
