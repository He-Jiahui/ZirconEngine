use std::collections::BTreeMap;

use super::super::runtime_state::HybridGiRuntimeState;

#[test]
fn rebuild_probe_child_probes_indexes_children_by_parent() {
    let mut state = HybridGiRuntimeState::default();
    state.probe_parent_probes = BTreeMap::from([(20, 10), (30, 10), (40, 20)]);

    state.rebuild_probe_child_probes();

    assert_eq!(state.probe_child_probes().get(&10).unwrap(), &vec![20, 30]);
    assert_eq!(state.probe_child_probes().get(&20).unwrap(), &vec![40]);
    assert_eq!(
        state.probe_descendant_ids_with_depth(10),
        vec![(20, 1), (40, 2), (30, 1)]
    );
}

#[test]
fn probe_descendant_ids_are_cycle_bounded() {
    let mut state = HybridGiRuntimeState::default();
    state.probe_parent_probes = BTreeMap::from([(10, 20), (20, 10)]);
    state.rebuild_probe_child_probes();

    assert_eq!(state.probe_descendant_ids(10), vec![20, 10]);
}
