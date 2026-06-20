use crate::core::diagnostics::{DiagnosticStore, DiagnosticStoreSnapshot};
use crate::scene::components::Name;
use crate::scene::ecs::{
    ChangeDetectionScanStats, ChangeTick, ChangeTickWindow, Changed, Component, ComponentTicks,
    QueryState, RemovedComponentsParam, SystemState, ECS_CHANGE_DETECTION_ADDED_MATCHES_DIAGNOSTIC,
    ECS_CHANGE_DETECTION_CHANGED_MATCHES_DIAGNOSTIC, ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC,
};
use crate::scene::{EntityId, World};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

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

    let changed = system.run(&mut world, |query| {
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
fn removed_components_tracks_recursive_despawn() {
    let mut world = World::empty();
    let parent = world
        .spawn((Name("Parent".to_string()), Health(1)))
        .unwrap();
    let child = world.spawn((Name("Child".to_string()), Health(2))).unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();

    type RemovedHealth = RemovedComponentsParam<Health>;
    let mut system = SystemState::<RemovedHealth>::new(&mut world).unwrap();
    assert!(system
        .run(&mut world, |mut removed| removed.read().collect::<Vec<_>>())
        .is_empty());

    world.remove_entity_recursive(parent);

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
    assert!(system
        .run(&mut world, |mut removed| removed.read().collect::<Vec<_>>())
        .is_empty());

    assert_eq!(world.remove::<Health>(entity).unwrap(), Some(Health(5)));

    let removed = system.run(&mut world, |mut removed| removed.read().collect::<Vec<_>>());
    assert_eq!(removed, vec![entity]);
}

#[test]
fn removed_component_reader_sizes_unread_entity_results() {
    let removal_source = include_str!("../ecs/removal.rs");

    assert!(removal_source.contains("let mut names = Vec::with_capacity(self.type_names.len());"));
    assert!(removal_source.contains("for name in self.type_names.values()"));
    assert!(removal_source.contains("names.push(name.as_str());"));
    assert!(!removal_source.contains("map(String::as_str)"));
    assert!(removal_source.contains("let Some(events) = self.events.get(&TypeId::of::<T>()) else"));
    assert!(removal_source.contains("return &[];"));
    assert!(removal_source.contains("events.as_slice()"));
    assert!(!removal_source.contains(".map(Vec::as_slice)"));
    assert!(removal_source.contains("let unread = &all[start..];"));
    assert!(removal_source.contains("Vec::with_capacity(unread.len())"));
    assert!(removal_source.contains("for event in unread"));
    assert!(removal_source.contains("entities.push(event.entity());"));
    assert!(!removal_source.contains("all[start..].iter().map(|event| event.entity()).collect()"));
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
        .and_then(|text| text.split("pub fn get_mut_at_tick_with_ticks").next())
        .expect("read ResourceStore::get_mut body");
    let ticked_get_mut_source = resource_store_source
        .split("pub fn get_mut_at_tick_with_ticks")
        .nth(1)
        .and_then(|text| text.split("pub fn remove<").next())
        .expect("read ResourceStore::get_mut_at_tick_with_ticks body");
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

    assert!(
        insert_source.contains("let ticks = if let Some(stored) = self.resources.get(&type_id)")
    );
    assert!(insert_source.contains("ticks.set_changed(tick);"));
    assert!(insert_source.contains("} else {\n            ComponentTicks::new(tick)\n        };"));
    assert!(insert_source.contains("let Some(stored) = self.resources.insert("));
    assert!(insert_source.contains("let Ok(boxed) = stored.value.downcast::<T>() else"));
    assert!(insert_source.contains("Some(*boxed)"));
    assert!(get_source.contains("let stored = self.resources.get(&TypeId::of::<T>())?;"));
    assert!(get_source.contains("stored.value.downcast_ref::<T>()"));
    assert!(get_mut_source.contains("let stored = self.resources.get_mut(&TypeId::of::<T>())?;"));
    assert!(get_mut_source.contains("stored.value.downcast_mut::<T>()"));
    assert!(
        ticked_get_mut_source.contains("let Some(value) = stored.value.downcast_mut::<T>() else")
    );
    assert!(ticked_get_mut_source.contains("Some((value, ticks))"));
    assert!(remove_source.contains("let stored = self.resources.remove(&TypeId::of::<T>())?;"));
    assert!(remove_source.contains("let Ok(boxed) = stored.value.downcast::<T>() else"));
    assert!(remove_source.contains("Some(*boxed)"));
    assert!(ticks_source.contains("let stored = self.resources.get(&TypeId::of::<T>())?;"));
    assert!(ticks_source.contains("Some(stored.ticks)"));
    assert!(!insert_source.contains(".map(|stored|"));
    assert!(!insert_source.contains(".unwrap_or_else(|| ComponentTicks::new(tick))"));
    assert!(!insert_source.contains(".and_then(|stored| stored.value.downcast::<T>().ok())"));
    assert!(!insert_source.contains(".map(|boxed| *boxed)"));
    assert!(!get_source.contains(".and_then(|stored| stored.value.downcast_ref::<T>())"));
    assert!(!get_mut_source.contains(".and_then(|stored| stored.value.downcast_mut::<T>())"));
    assert!(!ticked_get_mut_source
        .contains("stored.value.downcast_mut::<T>().map(|value| (value, ticks))"));
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
