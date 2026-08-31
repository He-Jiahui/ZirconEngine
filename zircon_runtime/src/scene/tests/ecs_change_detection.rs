use crate::core::diagnostics::{DiagnosticStore, DiagnosticStoreSnapshot};
use crate::scene::components::Name;
use crate::scene::ecs::{
    ChangeDetectionScanStats, ChangeTick, ChangeTickWindow, Changed, Component, ComponentTicks,
    ECS_CHANGE_DETECTION_ADDED_MATCHES_DIAGNOSTIC, ECS_CHANGE_DETECTION_CHANGED_MATCHES_DIAGNOSTIC,
    ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC, Mut, QueryState, RemovedComponentsParam,
    ResMutParam, ResParam, Resource, StorageType, SystemState,
};
use crate::scene::{EntityId, World};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct SparseHealth(u32);

impl Component for SparseHealth {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

#[derive(Debug, PartialEq, Eq)]
struct ChangeTrackedResource(u32);

impl Resource for ChangeTrackedResource {}

fn diagnostic_current(snapshot: &DiagnosticStoreSnapshot, path: &str) -> Option<f64> {
    snapshot
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .and_then(|series| series.current)
}

fn resource_store_source() -> &'static str {
    concat!(
        include_str!("../ecs/resource_store/mod.rs"),
        "\n",
        include_str!("../ecs/resource_store/stored_resource.rs"),
        "\n",
        include_str!("../ecs/resource_store/store.rs"),
    )
}

#[test]
fn changed_filter_includes_newly_added_components() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Changed".to_string()), Health(1)))
        .unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let changed = system.run(&mut world, |mut query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });

    assert_eq!(changed, vec![entity]);
}

#[test]
fn change_tick_comparison_survives_wraparound() {
    let last_run = ChangeTick::new(u64::MAX - 2);
    let this_run = ChangeTick::new(1);
    let wrapped_change = ChangeTick::new(u64::MAX);

    assert_eq!(ChangeTick::new(u64::MAX).next(), ChangeTick::ZERO);
    assert!(wrapped_change.is_newer_than(last_run, this_run));
    assert!(!last_run.is_newer_than(last_run, this_run));
    assert!(!ChangeTick::new(2).is_newer_than(last_run, this_run));
}

#[test]
fn tick_window_clamps_stale_ticks() {
    let this_run = ChangeTick::new(42);
    let stale_tick = ChangeTick::new(this_run.get().wrapping_sub(ChangeTick::MAX_CHANGE_AGE + 1));
    let window = ChangeTickWindow::new(stale_tick, this_run);

    assert_eq!(
        this_run.relative_to(window.last_run()).get(),
        ChangeTick::MAX_CHANGE_AGE
    );
    assert!(!ComponentTicks::new(stale_tick).is_added(window));
    assert!(!ComponentTicks::new(stale_tick).is_changed(window));

    let fresh_tick = ChangeTick::new(this_run.get().wrapping_sub(7));
    assert!(ComponentTicks::new(fresh_tick).is_changed(window));
}

#[test]
fn change_detection_scan_stats_record_mark_checks_and_diagnostics() {
    let window = ChangeTickWindow::new(ChangeTick::new(10), ChangeTick::new(20));
    let added_now = ComponentTicks::new(ChangeTick::new(19));
    let mut changed_now = ComponentTicks::new(ChangeTick::new(2));
    changed_now.set_changed(ChangeTick::new(18));
    let unchanged = ComponentTicks::new(ChangeTick::new(4));

    let mut added_stats = ChangeDetectionScanStats::default();
    assert!(added_stats.scan_added(added_now, window));
    assert!(!added_stats.scan_added(unchanged, window));
    assert_eq!(added_stats.scanned_marks, 2);
    assert_eq!(added_stats.added_matches, 1);
    assert_eq!(added_stats.changed_matches, 0);

    let mut changed_stats = ChangeDetectionScanStats::default();
    assert!(changed_stats.scan_changed(changed_now, window));
    assert!(!changed_stats.scan_changed(unchanged, window));
    assert_eq!(changed_stats.scanned_marks, 2);
    assert_eq!(changed_stats.changed_matches, 1);

    added_stats.merge(changed_stats);
    assert_eq!(added_stats.scanned_marks, 4);
    assert_eq!(added_stats.added_matches, 1);
    assert_eq!(added_stats.changed_matches, 1);

    let mut diagnostics = DiagnosticStore::default();
    added_stats.record_diagnostics(&mut diagnostics, 7);
    let snapshot = diagnostics.snapshot();
    assert_eq!(
        diagnostic_current(&snapshot, ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC),
        Some(4.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_CHANGE_DETECTION_ADDED_MATCHES_DIAGNOSTIC),
        Some(1.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_CHANGE_DETECTION_CHANGED_MATCHES_DIAGNOSTIC),
        Some(1.0)
    );
}

#[test]
fn change_detection_scan_skips_unmarked_archetypes() {
    let window = ChangeTickWindow::new(ChangeTick::new(10), ChangeTick::new(20));
    let unmarked = [
        ComponentTicks::new(ChangeTick::new(1)),
        ComponentTicks::new(ChangeTick::new(3)),
        ComponentTicks::new(ChangeTick::new(9)),
    ];
    let mut stats = ChangeDetectionScanStats::default();

    for ticks in unmarked.iter().copied() {
        assert!(!stats.scan_added(ticks, window));
        assert!(!stats.scan_changed(ticks, window));
    }

    assert_eq!(stats.scanned_marks, unmarked.len() as u64 * 2);
    assert_eq!(stats.added_matches, 0);
    assert_eq!(stats.changed_matches, 0);

    let mut diagnostics = DiagnosticStore::default();
    stats.record_diagnostics(&mut diagnostics, 8);
    let snapshot = diagnostics.snapshot();
    assert_eq!(
        diagnostic_current(&snapshot, ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC),
        Some((unmarked.len() * 2) as f64)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_CHANGE_DETECTION_ADDED_MATCHES_DIAGNOSTIC),
        Some(0.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_CHANGE_DETECTION_CHANGED_MATCHES_DIAGNOSTIC),
        Some(0.0)
    );
}

#[test]
fn mut_query_marks_table_components_only_after_mutable_access() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Tracked".to_string()), Health(10)))
        .unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    type MutableHealth = QueryState<Mut<'static, Health>>;
    let mut changed = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    let mut mutable = SystemState::<MutableHealth>::new(&mut world).unwrap();

    assert_eq!(
        changed.run(&mut world, |mut query| {
            query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
        }),
        vec![entity]
    );
    assert!(changed.run(&mut world, |mut query| query.iter().next().is_none()));

    mutable.run(&mut world, |mut query| {
        let health = query.get_mut(entity).unwrap();
        assert_eq!(health.0, 10);
    });
    assert!(changed.run(&mut world, |mut query| query.iter().next().is_none()));

    mutable.run(&mut world, |mut query| {
        let mut health = query.get_mut(entity).unwrap();
        health.0 += 1;
    });
    assert_eq!(
        changed.run(&mut world, |mut query| {
            query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
        }),
        vec![entity]
    );
}

#[test]
fn mut_query_marks_sparse_components_only_after_explicit_change() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("SparseTracked".to_string()), SparseHealth(10)))
        .unwrap();

    type MutableSparseHealth = QueryState<Mut<'static, SparseHealth>>;
    let mut mutable = SystemState::<MutableSparseHealth>::new(&mut world).unwrap();
    let before = world
        .component_change_ticks::<SparseHealth>(entity)
        .unwrap();

    mutable.run(&mut world, |mut query| {
        let health = query.get_mut(entity).unwrap();
        assert_eq!(health.0, 10);
    });
    assert_eq!(
        world.component_change_ticks::<SparseHealth>(entity),
        Some(before)
    );

    mutable.run(&mut world, |mut query| {
        let mut health = query.get_mut(entity).unwrap();
        health.set_changed();
    });
    assert_ne!(
        world.component_change_ticks::<SparseHealth>(entity),
        Some(before)
    );
}

#[test]
fn res_mut_marks_resources_only_after_mutable_access() {
    let mut world = World::empty();
    world.insert_resource(ChangeTrackedResource(10));

    let mut observed = SystemState::<ResParam<ChangeTrackedResource>>::new(&mut world).unwrap();
    let mut mutable = SystemState::<ResMutParam<ChangeTrackedResource>>::new(&mut world).unwrap();

    assert!(observed.run(&mut world, |resource| resource.is_changed()));
    assert!(!observed.run(&mut world, |resource| resource.is_changed()));

    mutable.run(&mut world, |resource| {
        assert_eq!(resource.0, 10);
    });
    assert!(!observed.run(&mut world, |resource| resource.is_changed()));

    mutable.run(&mut world, |mut resource| {
        resource.as_mut().0 += 1;
    });
    assert!(observed.run(&mut world, |resource| resource.is_changed()));
}

#[test]
fn cached_mut_query_fetch_does_not_mark_changed_until_the_wrapper_is_mutated() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("CachedTracked".to_string()), Health(10)))
        .unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    type CachedMutableHealth = QueryState<Mut<'static, Health>>;
    let mut changed = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    let mut cached_mutable = CachedMutableHealth::new(&mut world);

    assert_eq!(
        changed.run(&mut world, |mut query| {
            query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
        }),
        vec![entity]
    );
    assert_eq!(cached_mutable.cache_rebuilds(), 1);

    {
        let health = cached_mutable.get_mut(&mut world, entity).unwrap();
        assert_eq!(health.0, 10);
    }
    assert_eq!(cached_mutable.cache_rebuilds(), 1);
    assert!(changed.run(&mut world, |mut query| query.iter().next().is_none()));

    {
        let mut health = cached_mutable.get_mut(&mut world, entity).unwrap();
        health.0 += 1;
    }
    assert_eq!(
        changed.run(&mut world, |mut query| {
            query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
        }),
        vec![entity]
    );
}

#[test]
fn raw_world_mutable_access_marks_components_and_resources_eagerly() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("RawMutable".to_string()), Health(10)))
        .unwrap();
    world.insert_resource(ChangeTrackedResource(5));

    let component_before = world.component_change_ticks::<Health>(entity).unwrap();
    let resource_before = world
        .resource_change_ticks::<ChangeTrackedResource>()
        .unwrap();

    assert_eq!(world.get_mut::<Health>(entity).unwrap().0, 10);
    assert_eq!(
        world.get_resource_mut::<ChangeTrackedResource>().unwrap().0,
        5
    );

    assert_ne!(
        world.component_change_ticks::<Health>(entity),
        Some(component_before)
    );
    assert_ne!(
        world.resource_change_ticks::<ChangeTrackedResource>(),
        Some(resource_before)
    );
}

#[test]
fn optional_res_mut_fetch_is_not_a_change_until_it_mutates() {
    let mut world = World::empty();
    world.insert_resource(ChangeTrackedResource(10));

    type OptionalMutable = Option<ResMutParam<ChangeTrackedResource>>;
    let mut observed = SystemState::<ResParam<ChangeTrackedResource>>::new(&mut world).unwrap();
    let mut mutable = SystemState::<OptionalMutable>::new(&mut world).unwrap();

    assert!(observed.run(&mut world, |resource| resource.is_changed()));
    assert!(!observed.run(&mut world, |resource| resource.is_changed()));

    mutable.run(&mut world, |resource| {
        let resource = resource.expect("inserted resource must be available to optional ResMut");
        assert_eq!(resource.0, 10);
    });
    assert!(!observed.run(&mut world, |resource| resource.is_changed()));

    mutable.run(&mut world, |resource| {
        resource
            .expect("inserted resource must be available to optional ResMut")
            .set_changed();
    });
    assert!(observed.run(&mut world, |resource| resource.is_changed()));
}

#[test]
fn mut_wrapper_records_the_current_tick_across_wraparound_only_after_write() {
    let prior_tick = ChangeTick::new(u64::MAX - 1);
    let this_run = ChangeTick::new(1);
    let window = ChangeTickWindow::new(ChangeTick::new(u64::MAX - 2), this_run);
    let mut value = 10_u32;
    let mut ticks = ComponentTicks::new(prior_tick);

    {
        let mut mutable = Mut::new(&mut value, &mut ticks, this_run, window);
        assert_eq!(*mutable, 10);
        assert_eq!(mutable.last_changed(), prior_tick);

        *mutable += 1;
        assert_eq!(mutable.last_changed(), this_run);
    }

    assert_eq!(value, 11);
    assert_eq!(ticks.changed(), this_run);
    assert!(ticks.is_changed(window));
}

#[test]
fn mut_wrapper_into_inner_marks_the_current_tick() {
    let this_run = ChangeTick::new(14);
    let window = ChangeTickWindow::new(ChangeTick::new(13), this_run);
    let mut value = 10_u32;
    let mut ticks = ComponentTicks::new(ChangeTick::new(7));

    {
        let mutable = Mut::new(&mut value, &mut ticks, this_run, window);
        *mutable.into_inner() = 12;
    }

    assert_eq!(value, 12);
    assert_eq!(ticks.changed(), this_run);
    assert!(ticks.is_changed(window));
}

#[test]
fn removed_components_tracks_recursive_despawn() {
    let mut world = World::empty();
    let parent = world
        .spawn((Name("Parent".to_string()), Health(1)))
        .unwrap();
    let child = world.spawn((Name("Child".to_string()), Health(2))).unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();

    type RemovedHealth = RemovedComponentsParam<Health>;
    let mut system = SystemState::<RemovedHealth>::new(&mut world).unwrap();
    assert!(
        system
            .run(&mut world, |mut removed| removed.read().collect::<Vec<_>>())
            .is_empty()
    );

    let _batch = world.remove_entity_recursive(parent).unwrap();

    let removed = system.run(&mut world, |mut removed| removed.read().collect::<Vec<_>>());
    assert_eq!(removed, vec![child, parent]);
}

#[test]
fn component_removal_emits_removal_record_in_same_frame() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Removed".to_string()), Health(5)))
        .unwrap();

    type RemovedHealth = RemovedComponentsParam<Health>;
    let mut system = SystemState::<RemovedHealth>::new(&mut world).unwrap();
    assert!(
        system
            .run(&mut world, |mut removed| removed.read().collect::<Vec<_>>())
            .is_empty()
    );

    assert_eq!(world.remove::<Health>(entity).unwrap(), Some(Health(5)));

    let removed = system.run(&mut world, |mut removed| removed.read().collect::<Vec<_>>());
    assert_eq!(removed, vec![entity]);
}

#[test]
fn removed_component_reader_uses_bounded_queue_and_incremental_cursor() {
    let removal_source = include_str!("../ecs/removal.rs");

    assert!(removal_source.contains("let mut names = Vec::with_capacity(self.type_names.len());"));
    assert!(removal_source.contains("for name in self.type_names.values()"));
    assert!(removal_source.contains("names.push(name.as_str());"));
    assert!(!removal_source.contains("map(String::as_str)"));
    assert!(removal_source.contains("entries: VecDeque<RemovedComponentEntry>"));
    assert!(removal_source.contains("pub struct RemovedComponentRetention"));
    assert!(removal_source.contains("pub struct RemovedComponentWriteReceipt"));
    assert!(removal_source.contains("pub struct RemovedComponentReadIter"));
    assert!(
        removal_source.contains("self.reader.next_sequence = entry.sequence.saturating_add(1);")
    );
    assert!(!removal_source.contains("Vec<RemovedComponentEvent>"));
    assert!(!removal_source.contains("-> Vec<EntityId>"));
}

#[test]
fn resource_store_hot_paths_use_direct_branches() {
    let resource_store_source = resource_store_source();
    let insert_source = resource_store_source
        .split("pub fn insert_at_tick")
        .nth(1)
        .and_then(|text| text.split("pub fn get<").next())
        .expect("read ResourceStore::insert_at_tick body");
    let get_source = resource_store_source
        .split("pub fn get<")
        .nth(1)
        .and_then(|text| text.split("pub fn get_mut<").next())
        .expect("read ResourceStore::get body");
    let get_mut_source = resource_store_source
        .split("pub fn get_mut<")
        .nth(1)
        .and_then(|text| text.split("pub fn get_mut_with_ticks").next())
        .expect("read ResourceStore::get_mut body");
    let ticked_get_mut_source = resource_store_source
        .split("pub fn get_mut_with_ticks")
        .nth(1)
        .and_then(|text| text.split("pub fn remove<").next())
        .expect("read ResourceStore::get_mut_with_ticks body");
    let remove_source = resource_store_source
        .split("pub fn remove<")
        .nth(1)
        .and_then(|text| text.split("pub fn contains<").next())
        .expect("read ResourceStore::remove body");
    let ticks_source = resource_store_source
        .split("pub fn ticks<")
        .nth(1)
        .and_then(|text| text.split("pub fn len(&self)").next())
        .expect("read ResourceStore::ticks body");

    assert!(insert_source.contains("match self.resources.entry(type_id)"));
    assert!(insert_source.contains("Entry::Occupied(mut occupied)"));
    assert!(insert_source.contains("let stored = occupied.get_mut();"));
    assert!(insert_source.contains("ticks.set_changed(tick);"));
    assert!(insert_source.contains("std::mem::replace(&mut stored.value, Box::new(resource))"));
    assert!(insert_source.contains("Entry::Vacant(vacant)"));
    assert!(insert_source.contains("ticks: ComponentTicks::new(tick)"));
    assert!(insert_source.contains("let Ok(boxed) = previous.downcast::<T>() else"));
    assert!(insert_source.contains("Some(*boxed)"));
    assert!(get_source.contains("let stored = self.resources.get(&TypeId::of::<T>())?;"));
    assert!(get_source.contains("stored.value.downcast_ref::<T>()"));
    assert!(get_mut_source.contains("let stored = self.resources.get_mut(&TypeId::of::<T>())?;"));
    assert!(get_mut_source.contains("stored.value.downcast_mut::<T>()"));
    assert!(ticked_get_mut_source.contains("let StoredResource { value, ticks, .. } = stored;"));
    assert!(ticked_get_mut_source.contains("let value = value.downcast_mut::<T>()?;"));
    assert!(ticked_get_mut_source.contains("Some((value, ticks))"));
    assert!(remove_source.contains("let stored = self.resources.remove(&TypeId::of::<T>())?;"));
    assert!(remove_source.contains("let Ok(boxed) = stored.value.downcast::<T>() else"));
    assert!(remove_source.contains("Some(*boxed)"));
    assert!(ticks_source.contains("let stored = self.resources.get(&TypeId::of::<T>())?;"));
    assert!(ticks_source.contains("Some(stored.ticks)"));
    assert!(!insert_source.contains(".map(|stored|"));
    assert!(!insert_source.contains("self.resources.get(&type_id)"));
    assert!(!insert_source.contains("self.resources.insert("));
    assert!(!insert_source.contains(".unwrap_or_else(|| ComponentTicks::new(tick))"));
    assert!(!insert_source.contains(".and_then(|stored| stored.value.downcast::<T>().ok())"));
    assert!(!insert_source.contains(".map(|boxed| *boxed)"));
    assert!(!get_source.contains(".and_then(|stored| stored.value.downcast_ref::<T>())"));
    assert!(!get_mut_source.contains(".and_then(|stored| stored.value.downcast_mut::<T>())"));
    assert!(!ticked_get_mut_source.contains("set_changed"));
    assert!(
        !ticked_get_mut_source
            .contains("stored.value.downcast_mut::<T>().map(|value| (value, ticks))")
    );
    assert!(!remove_source.contains(".and_then(|stored| stored.value.downcast::<T>().ok())"));
    assert!(!remove_source.contains(".map(|boxed| *boxed)"));
    assert!(!ticks_source.contains(".map(|stored| stored.ticks)"));
}

#[test]
fn world_mutation_change_tick_uses_direct_active_tick_branch() {
    let change_detection_source = include_str!("../world/change_detection.rs");
    let mutation_change_tick = change_detection_source
        .split("pub(crate) fn mutation_change_tick")
        .nth(1)
        .and_then(|text| text.split("pub fn component_change_ticks").next())
        .expect("read World::mutation_change_tick body");

    assert!(
        mutation_change_tick.contains("if let Some(tick) = self.active_change_tick")
            && mutation_change_tick.contains("return tick;")
            && mutation_change_tick.contains("self.advance_change_tick()")
            && !mutation_change_tick.contains(".unwrap_or_else(|| self.advance_change_tick())"),
        "World::mutation_change_tick must branch directly on active system ticks instead of using an unwrap_or_else adapter"
    );
}
