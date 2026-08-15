use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::scene::ecs::component::TableColumnLayout;
use crate::scene::ecs::storage::StoredComponent;
use crate::scene::ecs::{ChangeTick, ComponentId, ComponentTicks};
use crate::scene::EntityId;

use super::{ArchetypeTable, ArchetypeTableError};

fn stored<T>(value: T) -> StoredComponent
where
    T: Send + Sync + 'static,
{
    Box::new(value)
}

fn ticks(value: u64) -> ComponentTicks {
    ComponentTicks::new(ChangeTick::new(value))
}

fn health_label_table(health: ComponentId, label: ComponentId) -> ArchetypeTable {
    ArchetypeTable::new([
        (label, TableColumnLayout::of::<&'static str>()),
        (health, TableColumnLayout::of::<i32>()),
    ])
}

fn health_table(health: ComponentId) -> ArchetypeTable {
    ArchetypeTable::new([(health, TableColumnLayout::of::<i32>())])
}

fn drop_spy_table(component: ComponentId) -> ArchetypeTable {
    ArchetypeTable::new([(component, TableColumnLayout::of::<DropSpy>())])
}

#[derive(Debug)]
struct DropSpy(Arc<AtomicUsize>);

impl Drop for DropSpy {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn archetype_table_keeps_each_column_row_aligned() {
    let health = ComponentId::new(1);
    let label = ComponentId::new(2);
    let mut table = health_label_table(health, label);

    let row = table
        .append_row(
            7,
            vec![
                (health, stored(10_i32), ticks(2)),
                (label, stored("hero"), ticks(3)),
            ],
        )
        .unwrap();

    assert_eq!(row, 0);
    assert_eq!(table.entities(), &[7]);
    assert_eq!(
        table.component_ids().collect::<Vec<_>>(),
        vec![health, label]
    );
    assert_eq!(table.get::<i32>(health, row), Some(&10));
    assert_eq!(table.get::<&'static str>(label, row), Some(&"hero"));
    assert_eq!(table.component_ticks(health, row), Some(ticks(2)));
    assert_eq!(table.component_ticks(label, row), Some(ticks(3)));
}

#[test]
fn archetype_table_replaces_and_marks_only_the_target_column() {
    let health = ComponentId::new(1);
    let label = ComponentId::new(2);
    let mut table = health_label_table(health, label);
    table
        .append_row(
            7,
            vec![
                (health, stored(10_i32), ticks(2)),
                (label, stored("hero"), ticks(3)),
            ],
        )
        .unwrap();

    *table
        .get_mut_at_tick::<i32>(health, 0, ChangeTick::new(9))
        .unwrap() = 20;
    let previous = table
        .replace(health, 0, stored(25_i32), ChangeTick::new(10))
        .unwrap();

    assert_eq!(*previous.downcast::<i32>().unwrap(), 20);
    assert_eq!(table.get::<i32>(health, 0), Some(&25));
    assert_eq!(table.get::<&'static str>(label, 0), Some(&"hero"));
    assert_eq!(table.component_ticks(health, 0), Some(ticks(10)));
    assert_eq!(table.component_ticks(label, 0), Some(ticks(3)));
}

#[test]
fn archetype_table_rejects_type_mismatches_before_writing_any_column() {
    let health = ComponentId::new(1);
    let mut table = health_table(health);

    assert_eq!(
        table.append_row(7, vec![(health, stored("not health"), ticks(2))]),
        Err(ArchetypeTableError::ComponentTypeMismatch {
            component_id: health,
            expected_type: std::any::type_name::<i32>(),
        })
    );
    assert!(table.is_empty());
}

#[test]
fn archetype_table_publishes_only_preflighted_rows() {
    let health = ComponentId::new(1);
    let label = ComponentId::new(2);
    let mut table = health_label_table(health, label);

    let row = table
        .preflight_row(vec![
            (health, stored(10_i32), ticks(2)),
            (label, stored("hero"), ticks(3)),
        ])
        .expect("complete, typed row should preflight");

    assert!(table.is_empty());

    let row = table.append_preflighted_row(7, row);
    assert_eq!(row, 0);
    assert_eq!(table.entities(), &[7]);
    assert_eq!(table.get::<i32>(health, row), Some(&10));
    assert_eq!(table.get::<&'static str>(label, row), Some(&"hero"));
}

#[test]
fn archetype_table_swap_remove_moves_all_columns_and_preserves_ticks() {
    let health = ComponentId::new(1);
    let label = ComponentId::new(2);
    let mut table = health_label_table(health, label);
    table
        .append_row(
            7,
            vec![
                (health, stored(10_i32), ticks(2)),
                (label, stored("hero"), ticks(3)),
            ],
        )
        .unwrap();
    table
        .append_row(
            8,
            vec![
                (health, stored(20_i32), ticks(4)),
                (label, stored("villain"), ticks(5)),
            ],
        )
        .unwrap();

    let taken = table.take_row(0, 7).unwrap();

    assert_eq!(taken.entity(), 7);
    assert_eq!(taken.swapped_entity(), Some(8));
    assert_eq!(table.entities(), &[8]);
    assert_eq!(table.get::<i32>(health, 0), Some(&20));
    assert_eq!(table.get::<&'static str>(label, 0), Some(&"villain"));
    assert_eq!(table.component_ticks(health, 0), Some(ticks(4)));
    assert_eq!(table.component_ticks(label, 0), Some(ticks(5)));

    let mut components = taken.into_components();
    let (health_value, health_ticks) = components.remove(&health).unwrap();
    let (label_value, label_ticks) = components.remove(&label).unwrap();
    assert_eq!(*health_value.downcast::<i32>().unwrap(), 10);
    assert_eq!(*label_value.downcast::<&'static str>().unwrap(), "hero");
    assert_eq!(health_ticks, ticks(2));
    assert_eq!(label_ticks, ticks(3));
    assert!(components.is_empty());
}

#[test]
fn archetype_table_swap_remove_moves_non_copy_bodies_without_double_drop() {
    let component = ComponentId::new(1);
    let drops = Arc::new(AtomicUsize::new(0));
    let mut table = drop_spy_table(component);
    table
        .append_row(
            7,
            vec![(component, stored(DropSpy(Arc::clone(&drops))), ticks(2))],
        )
        .unwrap();
    table
        .append_row(
            8,
            vec![(component, stored(DropSpy(Arc::clone(&drops))), ticks(3))],
        )
        .unwrap();

    let taken = table.take_row(0, 7).unwrap();
    assert_eq!(taken.swapped_entity(), Some(8));
    drop(taken);
    assert_eq!(drops.load(Ordering::Relaxed), 1);

    drop(table);
    assert_eq!(drops.load(Ordering::Relaxed), 2);
}

#[test]
fn archetype_table_rejects_bad_rows_without_releasing_existing_values() {
    let health = ComponentId::new(1);
    let label = ComponentId::new(2);
    let mut table = health_label_table(health, label);
    table
        .append_row(
            7,
            vec![
                (health, stored(10_i32), ticks(2)),
                (label, stored("hero"), ticks(3)),
            ],
        )
        .unwrap();

    assert_eq!(
        table.append_row(8, vec![(health, stored(20_i32), ticks(4))]),
        Err(ArchetypeTableError::MissingComponentColumn {
            component_id: label,
        })
    );
    assert!(matches!(
        table.take_row(1, 7),
        Err(ArchetypeTableError::RowOutOfBounds { row: 1, len: 1 })
    ));
    assert_eq!(table.entities(), &[7]);
    assert_eq!(table.get::<i32>(health, 0), Some(&10));
    assert_eq!(table.get::<&'static str>(label, 0), Some(&"hero"));
}
