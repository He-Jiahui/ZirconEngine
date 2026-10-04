use super::{QueryAccess, QueryAccessError};
use crate::scene::ecs::ComponentId;

fn assert_writes_are_mirrored(access: &QueryAccess) {
    assert!(access
        .writes()
        .iter()
        .all(|component_id| access.reads().binary_search(component_id).is_ok()));
}

#[test]
fn runtime60_batch_writes_remain_mirrored_in_reads() {
    let mut access = QueryAccess::default();
    for index in [7, 1, 4] {
        access
            .add_write(ComponentId::new(index))
            .expect("distinct writes should be accepted");
    }

    assert_eq!(
        access.reads(),
        [
            ComponentId::new(1),
            ComponentId::new(4),
            ComponentId::new(7)
        ]
    );
    assert_eq!(access.writes(), access.reads());
    assert_writes_are_mirrored(&access);
}

#[test]
fn runtime60_batch_merged_writes_remain_mirrored_in_reads() {
    let mut access = QueryAccess::default();
    access
        .add_write(ComponentId::new(9))
        .expect("initial write should be accepted");

    let mut other = QueryAccess::default();
    other.add_filter_read(ComponentId::new(8));
    other
        .add_write(ComponentId::new(4))
        .expect("distinct write should be accepted");
    access.merge_param_set_unchecked(&other);

    assert_writes_are_mirrored(&access);
    assert_eq!(
        access.reads(),
        [
            ComponentId::new(4),
            ComponentId::new(8),
            ComponentId::new(9)
        ]
    );
    assert_eq!(access.writes(), [ComponentId::new(4), ComponentId::new(9)]);
}

#[test]
fn runtime60_batch_repeated_write_keeps_the_conflict_diagnostic() {
    let component_id = ComponentId::new(12);
    let mut access = QueryAccess::default();
    access
        .add_write(component_id)
        .expect("first write should be accepted");

    assert_eq!(
        access.add_write(component_id),
        Err(QueryAccessError::ConflictingComponentAccess { component_id })
    );
    assert_eq!(access.reads(), [component_id]);
    assert_eq!(access.writes(), [component_id]);
}
