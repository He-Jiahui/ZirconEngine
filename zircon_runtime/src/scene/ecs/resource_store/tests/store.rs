use super::*;

#[derive(Debug, PartialEq, Eq)]
struct ResourceValue(u32);

#[test]
fn transferred_resource_rows_rebase_change_ticks_at_target_commit() {
    let source_tick = ChangeTick::new(13);
    let target_tick = ChangeTick::new(41);
    let mut source = ResourceStore::default();
    source.insert_at_tick(ResourceValue(7), source_tick);

    let rows = source.take_transferred_rows();
    assert!(source.is_empty());
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source_ticks(), ComponentTicks::new(source_tick));

    let mut target = ResourceStore::default();
    target.insert_transferred_rows(rows, target_tick);

    assert_eq!(target.get::<ResourceValue>(), Some(&ResourceValue(7)));
    assert_eq!(
        target.ticks::<ResourceValue>(),
        Some(ComponentTicks::new(target_tick))
    );
}
