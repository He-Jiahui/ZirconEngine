use super::*;

fn append_empty_row(index: &mut ArchetypeIndex, entity: EntityId) {
    let row = index
        .preflight_row(ArchetypeId::EMPTY, [])
        .expect("the empty archetype accepts an empty row");
    index.append_preflighted_row(ArchetypeId::EMPTY, entity, row);
}

#[test]
fn topology_snapshot_distinguishes_registered_signatures() {
    let baseline = ArchetypeIndex::new();
    let mut with_component = ArchetypeIndex::new();
    let component_id = ComponentId::new(37);

    with_component.id_or_insert(ArchetypeSignature::new([], [component_id]), []);

    assert_ne!(
        baseline.topology_snapshot(),
        with_component.topology_snapshot()
    );
    assert_ne!(baseline, with_component);
}

#[test]
fn topology_snapshot_distinguishes_entity_row_membership() {
    let baseline = ArchetypeIndex::new();
    let mut populated = ArchetypeIndex::new();

    append_empty_row(&mut populated, 101);

    assert_ne!(baseline.topology_snapshot(), populated.topology_snapshot());
    assert_ne!(baseline, populated);
}

#[test]
fn topology_snapshot_ignores_diagnostics_and_membership_history() {
    let baseline = ArchetypeIndex::new();
    let mut observed = ArchetypeIndex::new();

    append_empty_row(&mut observed, 202);
    let removed = observed
        .take_entity_row(ArchetypeId::EMPTY, 0, 202)
        .expect("the appended row remains addressable");
    drop(removed);
    let _ = observed.matching_archetypes(&[ComponentId::new(91)], &[]);

    assert_ne!(
        baseline.membership_generation(ArchetypeId::EMPTY),
        observed.membership_generation(ArchetypeId::EMPTY)
    );
    assert_ne!(baseline.performance_stats(), observed.performance_stats());
    assert_eq!(baseline.topology_snapshot(), observed.topology_snapshot());
    assert_eq!(baseline, observed);
}
