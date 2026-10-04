use super::HierarchyRowOverrides;
use crate::scene::WorldInspectionHierarchyRow;

#[test]
fn cloned_index_preserves_prior_rows_and_visits_each_current_replacement() {
    let mut rows = HierarchyRowOverrides::default();
    for index in [0, 1, 7, 63, 64, 1_024, usize::MAX] {
        rows.insert(index, row(index, "initial"));
    }
    let snapshot = rows.clone();

    rows.insert(63, row(63, "updated"));
    rows.insert(255, row(255, "new"));

    assert_eq!(snapshot.len(), 7);
    assert_eq!(
        snapshot.get(&63).map(|row| row.display_name.as_str()),
        Some("initial")
    );
    assert!(snapshot.get(&255).is_none());
    assert_eq!(rows.len(), 8);
    assert_eq!(
        rows.get(&63).map(|row| row.display_name.as_str()),
        Some("updated")
    );
    assert_eq!(
        rows.get(&255).map(|row| row.display_name.as_str()),
        Some("new")
    );
    let mut visited = Vec::new();
    rows.for_each(|index, _| visited.push(index));
    visited.sort_unstable();
    assert_eq!(visited, vec![0, 1, 7, 63, 64, 255, 1_024, usize::MAX]);
}

fn row(index: usize, display_name: &str) -> WorldInspectionHierarchyRow {
    WorldInspectionHierarchyRow {
        entity: index as u64,
        parent: None,
        depth: 0,
        display_name: display_name.to_string(),
        kind: "Empty".to_string(),
        subtree_hash: index as u64,
        active_in_hierarchy: true,
        has_children: false,
    }
}
