//! 批量及单项查询须保留请求顺序、重复项和系统运行时间窗中的 Changed 过滤。

use super::*;

#[test]
fn system_query_get_many_helpers_preserve_order_duplicates_and_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    let unique_first = UniqueEntityArray::new([first]).unwrap();

    let baseline = system.run(&mut world, |mut query| {
        (
            query
                .get_many([first, first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many([first, marker_only])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached([first, first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached_direct([first, first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_unique(unique_first)
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            UniqueEntityArray::new([first, first]),
            UniqueEntityArray::new([first, first]),
            UniqueEntityArray::new([first, first]),
        )
    });
    assert_eq!(
        baseline,
        (
            Ok([(first, 10), (first, 10)]),
            Err(QueryEntityError::QueryDoesNotMatch(marker_only)),
            Ok([(first, 10), (first, 10)]),
            Ok([(first, 10), (first, 10)]),
            Ok([(first, 10)]),
            Err(QueryEntityError::DuplicateEntity(first)),
            Err(QueryEntityError::DuplicateEntity(first)),
            Err(QueryEntityError::DuplicateEntity(first)),
        )
    );
    assert_eq!(system.state().cache_rebuilds(), 1);

    let unchanged = system.run(&mut world, |mut query| {
        (
            query
                .get_many([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached_direct([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
        )
    });
    assert_eq!(
        unchanged,
        (
            Err(QueryEntityError::QueryDoesNotMatch(first)),
            Err(QueryEntityError::QueryDoesNotMatch(first)),
            Err(QueryEntityError::QueryDoesNotMatch(first)),
        )
    );
    assert_eq!(system.state().cache_rebuilds(), 1);

    world.get_mut::<Health>(first).unwrap().0 = 11;
    let changed = system.run(&mut world, |mut query| {
        (
            query
                .get_many([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached_direct([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
        )
    });
    assert_eq!(
        changed,
        (Ok([(first, 11)]), Ok([(first, 11)]), Ok([(first, 11)]),)
    );
}

#[test]
fn system_query_iter_many_preserves_order_duplicates_and_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let requested = vec![marker_only, first, 999, first];
    let unique_first = UniqueEntityArray::new([first]).unwrap();
    let baseline = system.run(&mut world, |mut query| {
        (
            query
                .iter_many(&requested)
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
            query
                .iter_many_cached(&requested)
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
            query
                .iter_many_unique(unique_first)
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
            query
                .iter_many_unique_cached(unique_first)
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
        )
    });
    assert_eq!(
        baseline,
        (
            vec![(first, 10), (first, 10)],
            vec![(first, 10), (first, 10)],
            vec![(first, 10)],
            vec![(first, 10)]
        )
    );

    let unchanged = system.run(&mut world, |mut query| {
        (
            query
                .iter_many([first])
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
            query
                .iter_many_cached([first])
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
        )
    });
    assert!(unchanged.0.is_empty());
    assert!(unchanged.1.is_empty());

    world.get_mut::<Health>(first).unwrap().0 = 11;
    let changed = system.run(&mut world, |mut query| {
        (
            query
                .iter_many([first])
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
            query
                .iter_many_cached([first])
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
        )
    });
    assert_eq!(changed, (vec![(first, 11)], vec![(first, 11)]));
}

#[test]
fn system_query_single_helpers_report_zero_one_many_matches() {
    let mut world = World::empty();
    type PlayerHealth = QueryState<(EntityId, &'static Health), With<Player>>;
    let mut system = SystemState::<PlayerHealth>::new(&mut world).unwrap();

    let empty = system.run(&mut world, |mut query| {
        query.single().map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(empty, Err(QuerySingleError::NoEntities));

    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let one = system.run(&mut world, |mut query| {
        query.single().map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(one, Ok((player, 10)));

    let cached = system.run(&mut world, |mut query| {
        query
            .single_cached()
            .map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(cached, Ok((player, 10)));

    let cached_direct = system.run(&mut world, |mut query| {
        query
            .single_cached_direct()
            .map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(cached_direct, Ok((player, 10)));

    world
        .spawn((Name("Ally".to_string()), Health(7), Player))
        .unwrap();
    let many = system.run(&mut world, |mut query| {
        query.single().map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(many, Err(QuerySingleError::MultipleEntities));
}

use crate::scene::ecs::{ComponentTicks, Mut, StorageType};

#[derive(Debug, PartialEq, Eq)]
struct RetainedSparseHealth(u32);

impl Component for RetainedSparseHealth {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

trait RetainedParamValue: Component {
    fn new(value: u32) -> Self;
    fn value(&self) -> u32;
    fn set_value(&mut self, value: u32);
}

macro_rules! impl_retained_param_value {
    ($component:ty) => {
        impl RetainedParamValue for $component {
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
    };
}

impl_retained_param_value!(Health);
impl_retained_param_value!(RetainedSparseHealth);

fn retained_param_ticks<T: Component>(
    world: &World,
    entities: [EntityId; 3],
) -> [ComponentTicks; 3] {
    entities.map(|entity| world.component_change_ticks::<T>(entity).unwrap())
}

fn assert_retained_param_tick_changes(
    before: [ComponentTicks; 3],
    after: [ComponentTicks; 3],
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

fn retained_param_changes<T: RetainedParamValue, U: RetainedParamValue>(
    observer: &mut SystemState<(
        QueryState<(EntityId, &'static T), Changed<T>>,
        QueryState<(EntityId, &'static U), Changed<U>>,
    )>,
    world: &mut World,
) -> (Vec<u32>, Vec<u32>) {
    observer.run(world, |(mut tracked, mut eager)| {
        let mut tracked = tracked
            .iter()
            .map(|(_, value)| value.value())
            .collect::<Vec<_>>();
        let mut eager = eager
            .iter()
            .map(|(_, value)| value.value())
            .collect::<Vec<_>>();
        tracked.sort_unstable();
        eager.sort_unstable();
        (tracked, eager)
    })
}

fn assert_retained_mutable_params<T: RetainedParamValue, U: RetainedParamValue>() {
    let mut world = World::empty();
    let entities = [(10, 100), (20, 200), (30, 300)]
        .map(|(tracked, eager)| world.spawn((T::new(tracked), U::new(eager))).unwrap());
    let mut observer = SystemState::<(
        QueryState<(EntityId, &'static T), Changed<T>>,
        QueryState<(EntityId, &'static U), Changed<U>>,
    )>::new(&mut world)
    .unwrap();
    assert_eq!(
        retained_param_changes(&mut observer, &mut world),
        (vec![10, 20, 30], vec![100, 200, 300]),
    );
    assert_eq!(
        retained_param_changes(&mut observer, &mut world),
        (vec![], vec![]),
    );
    let mut writer =
        SystemState::<(QueryState<Mut<'static, T>>, QueryState<&'static mut U>)>::new(&mut world)
            .unwrap();

    let tracked_before = retained_param_ticks::<T>(&world, entities);
    let eager_before = retained_param_ticks::<U>(&world, entities);
    let generation_before = world.world_generation();
    writer.run(&mut world, |(mut tracked, mut eager)| {
        let [first, second, unused] = tracked.get_many_mut(entities).unwrap();
        assert_eq!(
            [first.value(), second.value(), unused.value()],
            [10, 20, 30]
        );
        // Plain access is eager even without a write; all tracked items stay live across it.
        let other = eager.get_mut(entities[0]).unwrap();
        assert_eq!(other.value(), 100);
        assert_eq!(
            [first.value(), second.value(), unused.value()],
            [10, 20, 30]
        );
    });
    let generation_after = world.world_generation();
    assert!(generation_after > generation_before);
    assert_eq!(retained_param_ticks::<T>(&world, entities), tracked_before);
    assert_retained_param_tick_changes(
        eager_before,
        retained_param_ticks::<U>(&world, entities),
        [true, false, false],
    );
    assert_eq!(
        retained_param_changes(&mut observer, &mut world),
        (vec![], vec![100]),
    );
    assert_eq!(
        retained_param_changes(&mut observer, &mut world),
        (vec![], vec![]),
    );

    let tracked_before = retained_param_ticks::<T>(&world, entities);
    let eager_before = retained_param_ticks::<U>(&world, entities);
    let generation_before = world.world_generation();
    writer.run(&mut world, |(mut tracked, mut eager)| {
        let [mut first, second, unused] = tracked.get_many_mut(entities).unwrap();
        let other = eager.get_mut(entities[1]).unwrap();
        other.set_value(201);
        // The earlier row and its delayed mutation sink are used after the other Param fetch.
        first.set_value(11);
        second.into_inner().set_value(21);
        assert_eq!([first.value(), unused.value()], [11, 30]);
    });
    let generation_after = world.world_generation();
    assert!(generation_after > generation_before);
    assert_eq!(
        entities.map(|entity| world.get::<T>(entity).unwrap().value()),
        [11, 21, 30]
    );
    assert_eq!(
        entities.map(|entity| world.get::<U>(entity).unwrap().value()),
        [100, 201, 300]
    );
    assert_retained_param_tick_changes(
        tracked_before,
        retained_param_ticks::<T>(&world, entities),
        [true, true, false],
    );
    assert_retained_param_tick_changes(
        eager_before,
        retained_param_ticks::<U>(&world, entities),
        [false, true, false],
    );
    assert_eq!(
        retained_param_changes(&mut observer, &mut world),
        (vec![11, 21], vec![201]),
    );
    assert_eq!(
        retained_param_changes(&mut observer, &mut world),
        (vec![], vec![]),
    );
}

#[test]
fn system_queries_keep_tracked_rows_live_across_disjoint_eager_params() {
    assert_retained_mutable_params::<Health, RetainedSparseHealth>();
    assert_retained_mutable_params::<RetainedSparseHealth, Health>();
}

#[derive(Clone, Copy, Debug)]
enum RetainedReadonlyOwner {
    Ordinary,
    Many,
    ManyCached,
    Combination,
    DirectCached,
    DirectManyCached,
}

fn assert_retained_readonly_cursor<'a, T, I>(
    mut cursor: I,
    entities: [EntityId; 3],
    mutate_other_param: impl FnOnce(),
) where
    T: RetainedParamValue,
    I: Iterator<Item = (EntityId, &'a T)>,
{
    let first = cursor.next().unwrap();
    assert_eq!((first.0, first.1.value()), (entities[0], 10));
    mutate_other_param();
    assert_eq!((first.0, first.1.value()), (entities[0], 10));
    let second = cursor.next().unwrap();
    let third = cursor.next().unwrap();
    assert_eq!((second.0, second.1.value()), (entities[1], 20));
    assert_eq!((third.0, third.1.value()), (entities[2], 30));
    assert!(cursor.next().is_none());
    assert_eq!(first.1.value(), 10);
}

fn assert_retained_readonly_pairs<'a, T, I>(
    mut cursor: I,
    entities: [EntityId; 3],
    mutate_other_param: impl FnOnce(),
) where
    T: RetainedParamValue,
    I: Iterator<Item = [(EntityId, &'a T); 2]>,
{
    let first = cursor.next().unwrap();
    mutate_other_param();
    assert_eq!(
        first.map(|(entity, value)| (entity, value.value())),
        [(entities[0], 10), (entities[1], 20)],
    );
    let remaining = cursor
        .map(|pair| pair.map(|(entity, value)| (entity, value.value())))
        .collect::<Vec<_>>();
    assert_eq!(
        remaining,
        vec![
            [(entities[0], 10), (entities[2], 30)],
            [(entities[1], 20), (entities[2], 30)],
        ],
    );
    assert_eq!([first[0].1.value(), first[1].1.value()], [10, 20]);
}

fn assert_retained_readonly_params<T: RetainedParamValue, U: RetainedParamValue>() {
    for owner in [
        RetainedReadonlyOwner::Ordinary,
        RetainedReadonlyOwner::Many,
        RetainedReadonlyOwner::ManyCached,
        RetainedReadonlyOwner::Combination,
        RetainedReadonlyOwner::DirectCached,
        RetainedReadonlyOwner::DirectManyCached,
    ] {
        let mut world = World::empty();
        let entities = [(10, 100), (20, 200), (30, 300)]
            .map(|(readonly, eager)| world.spawn((T::new(readonly), U::new(eager))).unwrap());
        let mut observer = SystemState::<(
            QueryState<(EntityId, &'static T), Changed<T>>,
            QueryState<(EntityId, &'static U), Changed<U>>,
        )>::new(&mut world)
        .unwrap();
        assert_eq!(
            retained_param_changes(&mut observer, &mut world),
            (vec![10, 20, 30], vec![100, 200, 300]),
        );
        assert_eq!(
            retained_param_changes(&mut observer, &mut world),
            (vec![], vec![]),
        );
        let readonly_before = retained_param_ticks::<T>(&world, entities);
        let eager_before = retained_param_ticks::<U>(&world, entities);
        let mut system = SystemState::<(
            QueryState<(EntityId, &'static T)>,
            QueryState<&'static mut U>,
        )>::new(&mut world)
        .unwrap();
        let generation_before = world.world_generation();
        system.run(&mut world, |(mut readonly, mut eager)| {
            let mutate_other_param = || eager.get_mut(entities[1]).unwrap().set_value(201);
            match owner {
                RetainedReadonlyOwner::Ordinary => {
                    assert_retained_readonly_cursor(readonly.iter(), entities, mutate_other_param)
                }
                RetainedReadonlyOwner::Many => assert_retained_readonly_cursor(
                    readonly.iter_many(entities),
                    entities,
                    mutate_other_param,
                ),
                RetainedReadonlyOwner::ManyCached => assert_retained_readonly_cursor(
                    readonly.iter_many_cached(entities),
                    entities,
                    mutate_other_param,
                ),
                RetainedReadonlyOwner::Combination => assert_retained_readonly_pairs(
                    readonly.iter_combinations::<2>(),
                    entities,
                    mutate_other_param,
                ),
                RetainedReadonlyOwner::DirectCached => assert_retained_readonly_cursor(
                    readonly.iter_cached_direct(),
                    entities,
                    mutate_other_param,
                ),
                RetainedReadonlyOwner::DirectManyCached => assert_retained_readonly_cursor(
                    readonly.iter_many_cached_direct(entities),
                    entities,
                    mutate_other_param,
                ),
            }
        });
        let generation_after = world.world_generation();
        assert!(generation_after > generation_before);
        assert_eq!(
            entities.map(|entity| world.get::<T>(entity).unwrap().value()),
            [10, 20, 30]
        );
        assert_eq!(
            entities.map(|entity| world.get::<U>(entity).unwrap().value()),
            [100, 201, 300]
        );
        assert_eq!(retained_param_ticks::<T>(&world, entities), readonly_before);
        assert_retained_param_tick_changes(
            eager_before,
            retained_param_ticks::<U>(&world, entities),
            [false, true, false],
        );
        assert_eq!(
            retained_param_changes(&mut observer, &mut world),
            (vec![], vec![201]),
        );
        assert_eq!(
            retained_param_changes(&mut observer, &mut world),
            (vec![], vec![]),
        );
    }
}

#[test]
fn system_queries_keep_six_readonly_cursors_live_across_disjoint_eager_params() {
    assert_retained_readonly_params::<Health, RetainedSparseHealth>();
    assert_retained_readonly_params::<RetainedSparseHealth, Health>();
}
