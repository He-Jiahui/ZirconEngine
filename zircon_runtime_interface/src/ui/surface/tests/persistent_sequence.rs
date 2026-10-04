use super::*;

struct NonClone(u32);

#[test]
fn owned_vector_construction_does_not_require_item_clones() {
    let sequence: UiPersistentSequence<_> = vec![NonClone(1), NonClone(2)].into();

    assert_eq!(sequence.len(), 2);
    assert_eq!(sequence[1].0, 2);
}

#[test]
fn one_item_mutation_clones_only_its_leaf_path() {
    let retained: UiPersistentSequence<_> = (0_u32..130).collect();
    let mut next = retained.clone();

    let (_, stats) = next
        .get_mut_with_stats(65)
        .map(|(item, stats)| {
            *item = 9_999;
            (item, stats)
        })
        .expect("mutable item");

    assert_eq!(retained[65], 65);
    assert_eq!(next[65], 9_999);
    assert_eq!(retained.shared_segment_count(&next), 2);
    assert_eq!(next.iter().copied().collect::<Vec<_>>().len(), 130);
    assert_eq!(stats.cloned_item_count, 64);
    assert_eq!(stats.cloned_segment_count, 1);
    assert_eq!(stats.cloned_directory_node_count, 1);
}

#[test]
fn serde_keeps_the_flat_wire_contract() {
    let sequence: UiPersistentSequence<_> = (0_u32..70).collect();
    let encoded = serde_json::to_string(&sequence).expect("serialize persistent sequence");
    let decoded: UiPersistentSequence<u32> =
        serde_json::from_str(&encoded).expect("deserialize persistent sequence");

    assert_eq!(decoded, sequence);
    assert_eq!(
        encoded,
        serde_json::to_string(&(0_u32..70).collect::<Vec<_>>()).unwrap()
    );
}

#[test]
fn unique_owner_mutation_uses_the_allocation_free_path() {
    let mut sequence: UiPersistentSequence<_> = (0_u32..130).collect();

    let (item, stats) = sequence
        .get_mut_with_stats(65)
        .expect("mutable unique-owner item");
    *item = 7_777;

    assert_eq!(sequence[65], 7_777);
    assert_eq!(stats, UiPersistentSequenceCowStats::default());
}
