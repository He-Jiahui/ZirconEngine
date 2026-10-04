use crate::scene::ecs::{
    ComponentId, QueryAccess, QueryAccessError, SystemParamAccess, SystemParamError,
};

#[test]
fn runtime60_batch_mixed_query_access_projects_read_only_ids_in_order() {
    let mut query = QueryAccess::default();
    query.add_filter_read(ComponentId::new(5));
    query
        .add_write(ComponentId::new(2))
        .expect("first write should be accepted");
    query.add_filter_read(ComponentId::new(1));
    query
        .add_write(ComponentId::new(7))
        .expect("second write should be accepted");

    let mut system = SystemParamAccess::default();
    system
        .add_query_access(&query)
        .expect("mixed query access should project");

    assert_eq!(
        system.component_access().reads(),
        [
            ComponentId::new(1),
            ComponentId::new(2),
            ComponentId::new(5),
            ComponentId::new(7)
        ]
    );
    assert_eq!(
        system.component_access().writes(),
        [ComponentId::new(2), ComponentId::new(7)]
    );
}

#[test]
fn runtime60_batch_interleaved_writes_are_skipped_by_the_monotonic_cursor() {
    let mut query = QueryAccess::default();
    for index in 0..8 {
        if index % 2 == 0 {
            query
                .add_write(ComponentId::new(index))
                .expect("distinct write should be accepted");
        } else {
            query.add_filter_read(ComponentId::new(index));
        }
    }

    let mut system = SystemParamAccess::default();
    system
        .add_query_access(&query)
        .expect("interleaved query access should project");

    assert_eq!(system.component_access().reads(), query.reads());
    assert_eq!(system.component_access().writes(), query.writes());
}

#[test]
fn runtime60_batch_projected_read_keeps_existing_write_conflicts() {
    let component_id = ComponentId::new(9);
    let mut existing = QueryAccess::default();
    existing
        .add_write(component_id)
        .expect("initial write should be accepted");
    let mut system = SystemParamAccess::default();
    system
        .add_query_access(&existing)
        .expect("initial query should project");

    let mut conflicting = QueryAccess::default();
    conflicting.add_filter_read(component_id);

    assert_eq!(
        system.add_query_access(&conflicting),
        Err(SystemParamError::Query(
            QueryAccessError::ConflictingComponentAccess { component_id }
        ))
    );
}
