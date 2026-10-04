use crate::scene::components::Name;
use crate::scene::ecs::{
    Changed, Component, Mut, QueryEntityError, QueryState, SystemState, UniqueEntityArray,
};
use crate::scene::World;

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

impl Component for Marker {}

#[test]
fn mutable_query_data_supports_read_only_projection_helpers() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    let mut query = world.query::<&mut Health>();
    assert_eq!(
        query
            .iter(&world)
            .map(|health| health.0)
            .collect::<Vec<_>>(),
        vec![10, 20]
    );
    assert_eq!(query.get(&world, first).map(|health| health.0), Ok(10));
    assert_eq!(
        query
            .iter_cached_direct(&world)
            .map(|health| health.0)
            .collect::<Vec<_>>(),
        vec![10, 20]
    );
    assert_eq!(
        query.get_cached_direct(&world, marker_only).map(|_| ()),
        Err(QueryEntityError::QueryDoesNotMatch(marker_only))
    );

    query.iter_mut(&mut world).for_each(|health| health.0 += 1);
    assert_eq!(world.get::<Health>(first), Some(&Health(11)));
    assert_eq!(world.get::<Health>(second), Some(&Health(21)));

    let mut tick_query = world.query::<Mut<'static, Health>>();
    assert_eq!(
        tick_query
            .iter_cached_direct(&world)
            .map(|health| (health.0, health.is_added(), health.is_changed()))
            .collect::<Vec<_>>(),
        vec![(11, true, true), (21, true, true)]
    );
}

#[test]
fn query_state_iter_mut_mutates_all_cached_matches_once() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    let mut query = world.query::<&mut Health>();
    let seen = query
        .iter_mut(&mut world)
        .map(|health| {
            let before = health.0;
            health.0 += 1;
            before
        })
        .collect::<Vec<_>>();

    assert_eq!(seen, vec![10, 20]);
    assert_eq!(world.get::<Health>(first), Some(&Health(11)));
    assert_eq!(world.get::<Health>(second), Some(&Health(21)));
    assert_eq!(world.get::<Health>(marker_only), None);
}

#[test]
fn query_state_iter_many_unique_mut_mutates_unique_targets_and_skips_mismatches() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    let mut query = world.query::<&mut Health>();
    let requested = UniqueEntityArray::new([marker_only, first, 999, second]).unwrap();
    let seen = query
        .iter_many_unique_mut(&mut world, requested)
        .map(|health| {
            let before = health.0;
            health.0 += 1;
            before
        })
        .collect::<Vec<_>>();

    assert_eq!(seen, vec![10, 20]);
    assert_eq!(world.get::<Health>(first), Some(&Health(11)));
    assert_eq!(world.get::<Health>(second), Some(&Health(21)));
    assert_eq!(
        UniqueEntityArray::new([first, first]),
        Err(QueryEntityError::DuplicateEntity(first))
    );
}

#[test]
fn query_state_get_many_unique_mut_mutates_all_targets_or_reports_mismatch() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    let mut query = world.query::<&mut Health>();
    let [left, right] = query
        .get_many_unique_mut(&mut world, UniqueEntityArray::new([first, second]).unwrap())
        .unwrap();
    left.0 += 1;
    right.0 += 10;
    assert_eq!(world.get::<Health>(first), Some(&Health(11)));
    assert_eq!(world.get::<Health>(second), Some(&Health(30)));

    assert_eq!(
        query
            .get_many_unique_mut(
                &mut world,
                UniqueEntityArray::new([first, marker_only]).unwrap(),
            )
            .map(|_| ()),
        Err(QueryEntityError::QueryDoesNotMatch(marker_only))
    );
}

#[test]
fn system_mutable_query_data_read_only_projection_uses_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();

    type ChangedHealth = QueryState<&'static mut Health, Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let baseline = system.run(&mut world, |mut query| {
        query.iter().map(|health| health.0).collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![10, 20]);

    let unchanged = system.run(&mut world, |mut query| query.iter().count());
    assert_eq!(unchanged, 0);

    world.get_mut::<Health>(second).unwrap().0 += 1;
    let changed = system.run(&mut world, |mut query| {
        query.iter().map(|health| health.0).collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![21]);
    assert_eq!(world.get::<Health>(first), Some(&Health(10)));
    assert_eq!(world.get::<Health>(second), Some(&Health(21)));
}

#[test]
fn system_mut_tick_wrapper_cached_direct_projection_preserves_ref_ticks() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();

    type ChangedHealth = QueryState<Mut<'static, Health>, Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let baseline = system.run(&mut world, |mut query| {
        query
            .iter_cached_direct()
            .map(|health| (health.0, health.is_added(), health.is_changed()))
            .collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![(10, true, true), (20, true, true)]);

    let unchanged = system.run(&mut world, |mut query| query.iter_cached_direct().count());
    assert_eq!(unchanged, 0);

    world.get_mut::<Health>(second).unwrap().0 += 1;
    let changed = system.run(&mut world, |mut query| {
        query
            .iter_cached_direct()
            .map(|health| (health.0, health.is_added(), health.is_changed()))
            .collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![(21, false, true)]);
    assert_eq!(world.get::<Health>(first), Some(&Health(10)));
    assert_eq!(world.get::<Health>(second), Some(&Health(21)));
}

#[test]
fn system_query_iter_mut_keeps_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();

    type ChangedHealth = QueryState<&'static mut Health, Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let baseline = system.run(&mut world, |mut query| {
        query
            .iter_mut()
            .map(|health| {
                health.0 += 1;
                health.0
            })
            .collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![11, 21]);

    let unchanged = system.run(&mut world, |mut query| query.iter_mut().count());
    assert_eq!(unchanged, 0);

    world.get_mut::<Health>(second).unwrap().0 += 1;
    let changed = system.run(&mut world, |mut query| {
        query
            .iter_mut()
            .map(|health| {
                health.0 += 10;
                health.0
            })
            .collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![32]);
    assert_eq!(world.get::<Health>(first), Some(&Health(11)));
    assert_eq!(world.get::<Health>(second), Some(&Health(32)));
}

#[test]
fn system_query_iter_many_unique_mut_is_iterator_and_keeps_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    type ChangedHealth = QueryState<&'static mut Health, Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    let requested = UniqueEntityArray::new([marker_only, first, 999, second]).unwrap();

    let baseline = system.run(&mut world, |mut query| {
        query
            .iter_many_unique_mut(requested)
            .map(|health| {
                health.0 += 1;
                health.0
            })
            .collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![11, 21]);

    let unchanged = system.run(&mut world, |mut query| {
        query.iter_many_unique_mut(requested).count()
    });
    assert_eq!(unchanged, 0);

    world.get_mut::<Health>(second).unwrap().0 += 1;
    let changed = system.run(&mut world, |mut query| {
        query
            .iter_many_unique_mut(requested)
            .map(|health| {
                health.0 += 10;
                health.0
            })
            .collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![32]);
}

#[test]
fn system_query_get_many_unique_mut_uses_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();

    type ChangedHealth = QueryState<&'static mut Health, Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    let requested = UniqueEntityArray::new([first, second]).unwrap();

    let baseline = system.run(&mut world, |mut query| {
        let [left, right] = query.get_many_unique_mut(requested).unwrap();
        left.0 += 1;
        right.0 += 10;
        [left.0, right.0]
    });
    assert_eq!(baseline, [11, 30]);

    let unchanged = system.run(&mut world, |mut query| {
        query.get_many_unique_mut(requested).map(|_| ())
    });
    assert_eq!(unchanged, Err(QueryEntityError::QueryDoesNotMatch(first)));

    world.get_mut::<Health>(first).unwrap().0 += 1;
    world.get_mut::<Health>(second).unwrap().0 += 1;
    let changed = system.run(&mut world, |mut query| {
        let [left, right] = query.get_many_unique_mut(requested).unwrap();
        left.0 += 10;
        right.0 += 100;
        [left.0, right.0]
    });
    assert_eq!(changed, [22, 131]);
}

#[test]
fn unsafe_custom_mutable_query_preserves_real_borrows_and_change_ticks() {
    use crate::scene::ecs::{
        ChangeDetectionScanStats, ChangeTickWindow, ComponentStorageLocation, QueryAccess,
        QueryAccessError, QueryDataAccess, QueryFilter, QueryMutData,
    };
    use crate::scene::EntityId;

    type Base = Mut<'static, Health>;
    struct FaithfulMutable;
    struct CandidateLocal;

    // SAFETY: all access and matching are delegated to the same component's real Base.
    unsafe impl QueryDataAccess for FaithfulMutable {
        fn update_access(
            world: &mut World,
            access: &mut QueryAccess,
        ) -> Result<(), QueryAccessError> {
            <Base as QueryDataAccess>::update_access(world, access)
        }
        fn matches_data(world: &World, entity: EntityId) -> bool {
            <Base as QueryDataAccess>::matches_data(world, entity)
        }
        fn matches_component_locations(
            world: &World,
            entity: EntityId,
            locations: &[ComponentStorageLocation],
        ) -> bool {
            <Base as QueryDataAccess>::matches_component_locations(world, entity, locations)
        }
    }
    // SAFETY: Base receives the unchanged entity, locations and tick window on every route.
    unsafe impl QueryMutData for FaithfulMutable {
        type Item<'world> = Mut<'world, Health>;
        unsafe fn fetch_mut<'world>(
            world: *mut World,
            entity: EntityId,
        ) -> Option<Self::Item<'world>> {
            unsafe { <Base as QueryMutData>::fetch_mut(world, entity) }
        }
        unsafe fn fetch_mut_with_ticks<'world>(
            world: *mut World,
            entity: EntityId,
            ticks: ChangeTickWindow,
        ) -> Option<Self::Item<'world>> {
            unsafe { <Base as QueryMutData>::fetch_mut_with_ticks(world, entity, ticks) }
        }
        unsafe fn fetch_mut_with_component_locations<'world>(
            world: *mut World,
            entity: EntityId,
            locations: &[ComponentStorageLocation],
            ticks: ChangeTickWindow,
        ) -> Option<Self::Item<'world>> {
            unsafe {
                <Base as QueryMutData>::fetch_mut_with_component_locations(
                    world, entity, locations, ticks,
                )
            }
        }
    }
    // SAFETY: the unit filter performs no undeclared read and retains every supplied candidate.
    unsafe impl QueryFilter for CandidateLocal {
        fn update_access(
            world: &mut World,
            access: &mut QueryAccess,
        ) -> Result<(), QueryAccessError> {
            <() as QueryFilter>::update_access(world, access)
        }
        fn matches(world: &World, entity: EntityId, ticks: ChangeTickWindow) -> bool {
            <() as QueryFilter>::matches(world, entity, ticks)
        }
        fn matches_component_locations(
            world: &World,
            entity: EntityId,
            locations: &[ComponentStorageLocation],
            ticks: ChangeTickWindow,
        ) -> bool {
            <() as QueryFilter>::matches_component_locations(world, entity, locations, ticks)
        }
        fn matches_component_locations_with_stats(
            world: &World,
            entity: EntityId,
            locations: &[ComponentStorageLocation],
            ticks: ChangeTickWindow,
            stats: &mut ChangeDetectionScanStats,
        ) -> bool {
            <() as QueryFilter>::matches_component_locations_with_stats(
                world, entity, locations, ticks, stats,
            )
        }
    }

    type ChangedHealth = QueryState<Mut<'static, Health>, Changed<Health>>;
    fn changed_values(system: &mut SystemState<ChangedHealth>, world: &mut World) -> Vec<u32> {
        system.run(world, |mut query| {
            let mut values = query
                .iter_cached_direct()
                .map(|health| health.0)
                .collect::<Vec<_>>();
            values.sort_unstable();
            values
        })
    }

    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let mut changed = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    assert_eq!(changed_values(&mut changed, &mut world), vec![10, 20]);
    assert!(changed_values(&mut changed, &mut world).is_empty());
    let mut query = world.query_filtered::<FaithfulMutable, CandidateLocal>();

    {
        let [mut left, mut right] = query.get_many_mut(&mut world, [first, second]).unwrap();
        left.0 += 1;
        right.0 += 10;
    }
    assert_eq!(world.get::<Health>(first), Some(&Health(11)));
    assert_eq!(world.get::<Health>(second), Some(&Health(30)));
    assert_eq!(changed_values(&mut changed, &mut world), vec![11, 30]);
    assert!(changed_values(&mut changed, &mut world).is_empty());

    assert_eq!(
        query.get_many_mut(&mut world, [first, first]).map(|_| ()),
        Err(QueryEntityError::AliasedMutability(first)),
    );
    drop(query.get_many_mut(&mut world, [first, second]).unwrap());
    assert!(changed_values(&mut changed, &mut world).is_empty());

    {
        let requested = vec![first, first, second];
        let mut iter = query.iter_many_mut(&mut world, &requested);
        let mut before = Vec::new();
        while let Some(mut health) = iter.fetch_next() {
            before.push(health.0);
            health.0 += 1;
        }
        assert_eq!(before, vec![11, 12, 30]);
    }
    assert_eq!(world.get::<Health>(first), Some(&Health(13)));
    assert_eq!(world.get::<Health>(second), Some(&Health(31)));
    assert_eq!(changed_values(&mut changed, &mut world), vec![13, 31]);
    assert!(changed_values(&mut changed, &mut world).is_empty());

    let mut builtin = world.query::<Mut<'static, Health>>();
    builtin.get_mut(&mut world, second).unwrap().0 += 1;
    assert_eq!(changed_values(&mut changed, &mut world), vec![32]);
    assert!(changed_values(&mut changed, &mut world).is_empty());
}

#[derive(Debug, PartialEq, Eq)]
struct SparseHealth(u32);

impl Component for SparseHealth {
    const STORAGE_TYPE: crate::scene::ecs::StorageType = crate::scene::ecs::StorageType::SparseSet;
}

trait TrackedRowValue: Component {
    fn new(value: u32) -> Self;
    fn value(&self) -> u32;
    fn set_value(&mut self, value: u32);
}

impl TrackedRowValue for Health {
    fn new(value: u32) -> Self {
        Self(value)
    }
    fn value(&self) -> u32 {
        self.0
    }
    fn set_value(&mut self, value: u32) {
        self.0 = value;
    }
}

impl TrackedRowValue for SparseHealth {
    fn new(value: u32) -> Self {
        Self(value)
    }
    fn value(&self) -> u32 {
        self.0
    }
    fn set_value(&mut self, value: u32) {
        self.0 = value;
    }
}

fn stored_row_values<T: TrackedRowValue>(
    world: &World,
    entities: [crate::scene::EntityId; 3],
) -> [u32; 3] {
    entities.map(|entity| world.get::<T>(entity).unwrap().value())
}

fn row_ticks<T: Component>(
    world: &World,
    entities: [crate::scene::EntityId; 3],
) -> [crate::scene::ecs::ComponentTicks; 3] {
    entities.map(|entity| world.component_change_ticks::<T>(entity).unwrap())
}

fn changed_row_values<T: TrackedRowValue>(
    system: &mut SystemState<QueryState<Mut<'static, T>, Changed<T>>>,
    world: &mut World,
) -> Vec<u32> {
    system.run(world, |mut query| {
        let mut values = query
            .iter_cached_direct()
            .map(|row| row.value())
            .collect::<Vec<_>>();
        values.sort_unstable();
        values
    })
}

fn assert_row_tick_changes(
    before: [crate::scene::ecs::ComponentTicks; 3],
    after: [crate::scene::ecs::ComponentTicks; 3],
    changed: [bool; 3],
) {
    for index in 0..3 {
        assert_eq!(after[index].added(), before[index].added());
        if changed[index] {
            assert_ne!(after[index].changed(), before[index].changed());
        } else {
            assert_eq!(after[index], before[index]);
        }
    }
}

fn assert_retained_tracked_rows<T: TrackedRowValue>() {
    let mut world = World::empty();
    let entities = [10, 20, 30].map(|value| world.spawn((T::new(value),)).unwrap());
    let mut changed =
        SystemState::<QueryState<Mut<'static, T>, Changed<T>>>::new(&mut world).unwrap();
    assert_eq!(
        changed_row_values(&mut changed, &mut world),
        vec![10, 20, 30]
    );
    assert!(changed_row_values(&mut changed, &mut world).is_empty());
    let mut query = world.query::<Mut<'static, T>>();

    let before = row_ticks::<T>(&world, entities);
    {
        let [first, second, third] = query.get_many_mut(&mut world, entities).unwrap();
        assert_eq!([first.value(), second.value(), third.value()], [10, 20, 30]);
    }
    assert_eq!(row_ticks::<T>(&world, entities), before);
    assert!(changed_row_values(&mut changed, &mut world).is_empty());

    let before = row_ticks::<T>(&world, entities);
    {
        let [mut first, mut second, third] = query.get_many_mut(&mut world, entities).unwrap();
        // All three real items have been fetched before either earlier item is used mutably.
        assert_eq!([first.value(), second.value(), third.value()], [10, 20, 30]);
        first.set_value(11);
        second.set_value(21);
        assert_eq!(third.value(), 30);
    }
    assert_eq!(stored_row_values::<T>(&world, entities), [11, 21, 30]);
    assert_row_tick_changes(
        before,
        row_ticks::<T>(&world, entities),
        [true, true, false],
    );
    assert_eq!(changed_row_values(&mut changed, &mut world), vec![11, 21]);
    assert!(changed_row_values(&mut changed, &mut world).is_empty());

    let before = row_ticks::<T>(&world, entities);
    {
        let mut iter = query.iter_mut(&mut world);
        let mut first = iter.next().unwrap();
        let mut second = iter.next().unwrap();
        let third = iter.next().unwrap();
        assert!(iter.next().is_none());
        assert_eq!([first.value(), second.value(), third.value()], [11, 21, 30]);
        first.set_value(12);
        second.set_value(22);
        assert_eq!(third.value(), 30);
    }
    assert_eq!(stored_row_values::<T>(&world, entities), [12, 22, 30]);
    assert_row_tick_changes(
        before,
        row_ticks::<T>(&world, entities),
        [true, true, false],
    );
    assert_eq!(changed_row_values(&mut changed, &mut world), vec![12, 22]);
    assert!(changed_row_values(&mut changed, &mut world).is_empty());

    let before = row_ticks::<T>(&world, entities);
    {
        let requested = UniqueEntityArray::new([entities[2], entities[0], entities[1]]).unwrap();
        let mut iter = query.iter_many_unique_mut(&mut world, requested);
        let mut third = iter.next().unwrap();
        let mut first = iter.next().unwrap();
        let second = iter.next().unwrap();
        assert!(iter.next().is_none());
        assert_eq!([first.value(), second.value(), third.value()], [12, 22, 30]);
        first.set_value(13);
        third.set_value(31);
        assert_eq!(second.value(), 22);
    }
    assert_eq!(stored_row_values::<T>(&world, entities), [13, 22, 31]);
    assert_row_tick_changes(
        before,
        row_ticks::<T>(&world, entities),
        [true, false, true],
    );
    assert_eq!(changed_row_values(&mut changed, &mut world), vec![13, 31]);
    assert!(changed_row_values(&mut changed, &mut world).is_empty());

    let before = row_ticks::<T>(&world, entities);
    {
        let mut combinations = query.iter_combinations_mut::<2>(&mut world);
        {
            let [mut first, mut second] = combinations.fetch_next().unwrap();
            assert_eq!([first.value(), second.value()], [13, 22]);
            first.set_value(14);
            second.set_value(23);
        }
        {
            let [first, third] = combinations.fetch_next().unwrap();
            assert_eq!([first.value(), third.value()], [14, 31]);
        }
        {
            let [second, third] = combinations.fetch_next().unwrap();
            assert_eq!([second.value(), third.value()], [23, 31]);
        }
        assert!(combinations.fetch_next().is_none());
    }
    assert_eq!(stored_row_values::<T>(&world, entities), [14, 23, 31]);
    assert_row_tick_changes(
        before,
        row_ticks::<T>(&world, entities),
        [true, true, false],
    );
    assert_eq!(changed_row_values(&mut changed, &mut world), vec![14, 23]);
    assert!(changed_row_values(&mut changed, &mut world).is_empty());

    assert_eq!(
        query
            .get_many_mut(&mut world, [entities[0], entities[0]])
            .map(|_| ()),
        Err(QueryEntityError::AliasedMutability(entities[0])),
    );
    let before = row_ticks::<T>(&world, entities);
    {
        let requested = [entities[0], entities[0], entities[2]];
        let mut iter = query.iter_many_mut(&mut world, &requested);
        {
            let first = iter.fetch_next().unwrap();
            assert_eq!(first.value(), 14);
            first.into_inner().set_value(15);
        }
        {
            let mut repeated = iter.fetch_next().unwrap();
            assert_eq!(repeated.value(), 15);
            repeated.set_value(16);
        }
        {
            let third = iter.fetch_next().unwrap();
            assert_eq!(third.value(), 31);
        }
        assert!(iter.fetch_next().is_none());
    }
    assert_eq!(stored_row_values::<T>(&world, entities), [16, 23, 31]);
    assert_row_tick_changes(
        before,
        row_ticks::<T>(&world, entities),
        [true, false, false],
    );
    assert_eq!(changed_row_values(&mut changed, &mut world), vec![16]);
    assert!(changed_row_values(&mut changed, &mut world).is_empty());
}

#[test]
fn tracked_mutable_query_keeps_table_row_items_live_across_fetches() {
    assert_retained_tracked_rows::<Health>();
}

#[test]
fn tracked_mutable_query_keeps_sparse_row_items_live_across_fetches() {
    assert_retained_tracked_rows::<SparseHealth>();
}

#[test]
fn custom_mixed_mutable_query_keeps_tracked_sink_live_across_plain_fetches() {
    use crate::scene::ecs::{
        ChangeTickWindow, ComponentStorageLocation, QueryAccess, QueryAccessError, QueryDataAccess,
        QueryMutData,
    };
    use crate::scene::EntityId;

    type Plain = &'static mut Health;
    type Tracked = Mut<'static, SparseHealth>;
    struct Mixed;

    // SAFETY: both distinct component declarations and candidate-local matching
    // are delegated faithfully; protected component checks remain in the builtins.
    unsafe impl QueryDataAccess for Mixed {
        fn update_access(
            world: &mut World,
            access: &mut QueryAccess,
        ) -> Result<(), QueryAccessError> {
            <Plain as QueryDataAccess>::update_access(world, access)?;
            <Tracked as QueryDataAccess>::update_access(world, access)
        }
        fn matches_data(world: &World, entity: EntityId) -> bool {
            <Plain as QueryDataAccess>::matches_data(world, entity)
                && <Tracked as QueryDataAccess>::matches_data(world, entity)
        }
        fn matches_component_locations(
            world: &World,
            entity: EntityId,
            locations: &[ComponentStorageLocation],
        ) -> bool {
            <Plain as QueryDataAccess>::matches_component_locations(world, entity, locations)
                && <Tracked as QueryDataAccess>::matches_component_locations(
                    world, entity, locations,
                )
        }
    }
    // SAFETY: this is a custom item, not a builtin mutable tuple. Each delegated
    // fetch uses the same entity/window/locations and disjoint Health/SparseHealth
    // leaves; neither member replaces candidates or retains a World/container borrow.
    unsafe impl QueryMutData for Mixed {
        type Item<'world> = (&'world mut Health, Mut<'world, SparseHealth>);
        unsafe fn fetch_mut<'world>(
            world: *mut World,
            entity: EntityId,
        ) -> Option<Self::Item<'world>> {
            Some((
                unsafe { <Plain as QueryMutData>::fetch_mut(world, entity)? },
                unsafe { <Tracked as QueryMutData>::fetch_mut(world, entity)? },
            ))
        }
        unsafe fn fetch_mut_with_ticks<'world>(
            world: *mut World,
            entity: EntityId,
            ticks: ChangeTickWindow,
        ) -> Option<Self::Item<'world>> {
            Some((
                unsafe { <Plain as QueryMutData>::fetch_mut_with_ticks(world, entity, ticks)? },
                unsafe { <Tracked as QueryMutData>::fetch_mut_with_ticks(world, entity, ticks)? },
            ))
        }
        unsafe fn fetch_mut_with_component_locations<'world>(
            world: *mut World,
            entity: EntityId,
            locations: &[ComponentStorageLocation],
            ticks: ChangeTickWindow,
        ) -> Option<Self::Item<'world>> {
            Some((
                unsafe {
                    <Plain as QueryMutData>::fetch_mut_with_component_locations(
                        world, entity, locations, ticks,
                    )?
                },
                unsafe {
                    <Tracked as QueryMutData>::fetch_mut_with_component_locations(
                        world, entity, locations, ticks,
                    )?
                },
            ))
        }
    }

    let mut world = World::empty();
    let entities = [(10, 100), (20, 200), (30, 300)]
        .map(|(plain, tracked)| world.spawn((Health(plain), SparseHealth(tracked))).unwrap());
    let mut plain_changed =
        SystemState::<QueryState<Mut<'static, Health>, Changed<Health>>>::new(&mut world).unwrap();
    let mut tracked_changed = SystemState::<
        QueryState<Mut<'static, SparseHealth>, Changed<SparseHealth>>,
    >::new(&mut world)
    .unwrap();
    assert_eq!(
        changed_row_values(&mut plain_changed, &mut world),
        vec![10, 20, 30]
    );
    assert_eq!(
        changed_row_values(&mut tracked_changed, &mut world),
        vec![100, 200, 300]
    );
    assert!(changed_row_values(&mut plain_changed, &mut world).is_empty());
    assert!(changed_row_values(&mut tracked_changed, &mut world).is_empty());
    let before_plain = row_ticks::<Health>(&world, entities);
    let before_tracked = row_ticks::<SparseHealth>(&world, entities);
    let mut query = world.query::<Mixed>();
    {
        let mut iter = query.iter_mut(&mut world);
        let (first_plain, mut first_tracked) = iter.next().unwrap();
        // The first tracked recorder stays live while later plain eager fetches record.
        let (second_plain, mut second_tracked) = iter.next().unwrap();
        let (third_plain, third_tracked) = iter.next().unwrap();
        assert!(iter.next().is_none());
        assert_eq!([first_plain.0, second_plain.0, third_plain.0], [10, 20, 30]);
        assert_eq!(
            [first_tracked.0, second_tracked.0, third_tracked.0],
            [100, 200, 300]
        );
        first_plain.0 += 1;
        second_plain.0 += 2;
        first_tracked.0 += 1;
        second_tracked.0 += 2;
        assert_eq!((third_plain.0, third_tracked.0), (30, 300));
    }
    assert_eq!(stored_row_values::<Health>(&world, entities), [11, 22, 30]);
    assert_eq!(
        stored_row_values::<SparseHealth>(&world, entities),
        [101, 202, 300]
    );
    // Plain mutable access is eager, including the read-only use of its third row.
    assert_row_tick_changes(
        before_plain,
        row_ticks::<Health>(&world, entities),
        [true, true, true],
    );
    assert_row_tick_changes(
        before_tracked,
        row_ticks::<SparseHealth>(&world, entities),
        [true, true, false],
    );
    assert_eq!(
        changed_row_values(&mut plain_changed, &mut world),
        vec![11, 22, 30]
    );
    assert_eq!(
        changed_row_values(&mut tracked_changed, &mut world),
        vec![101, 202]
    );
    assert!(changed_row_values(&mut plain_changed, &mut world).is_empty());
    assert!(changed_row_values(&mut tracked_changed, &mut world).is_empty());
}
