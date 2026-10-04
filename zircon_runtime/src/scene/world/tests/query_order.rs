use super::*;

#[test]
fn visits_matching_archetypes_in_stable_world_order_after_moves_and_removal() {
    let first = ArchetypeId::new(1);
    let second = ArchetypeId::new(2);
    let mut index = StableQueryOrderIndex::default();
    for entity in [10, 20, 30, 40] {
        index.register(entity, InternalEntity::new(entity as u32, 0));
    }
    index.move_to(10, EntityLocation::new(first, 0));
    index.move_to(20, EntityLocation::new(second, 0));
    index.move_to(30, EntityLocation::new(first, 1));
    index.move_to(40, EntityLocation::new(second, 1));
    index.move_to(10, EntityLocation::new(second, 2));
    index.update_row(30, 0);
    index.remove(30);

    let mut visited = Vec::new();
    index.visit_matching(&[first, second], |location| {
        visited.push((location.stable_id, location.location.table_row))
    });

    assert_eq!(visited, vec![(10, 2), (20, 0), (40, 1)]);
}

#[test]
fn rebuild_discards_old_membership_and_restores_the_supplied_world_order() {
    let first = ArchetypeId::new(1);
    let second = ArchetypeId::new(2);
    let mut index = StableQueryOrderIndex::default();
    index.register(1, InternalEntity::new(1, 0));
    index.move_to(1, EntityLocation::new(first, 0));

    index.rebuild([
        (30, InternalEntity::new(30, 0)),
        (10, InternalEntity::new(10, 0)),
        (20, InternalEntity::new(20, 0)),
    ]);
    index.move_to(30, EntityLocation::new(second, 0));
    index.move_to(10, EntityLocation::new(first, 0));
    index.move_to(20, EntityLocation::new(second, 1));

    let mut visited = Vec::new();
    index.visit_matching(&[second, first], |location| {
        visited.push(location.stable_id)
    });

    assert_eq!(visited, vec![30, 10, 20]);
}
