use super::*;
use crate::core::resource::{
    ResourceEvent, ResourceEventKind, ResourceLocator, approximate_event_bytes,
};
use crate::scene::PreparedDynamicSceneSpawn;

#[test]
fn dynamic_scene_asset_reload_event_bytes_are_cumulative_within_one_tick() {
    let fixture = SceneReloadFixture::new("asset_reload_cumulative_event_bytes");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let shader_id = ResourceId::from_stable_label("cumulative event budget shader");
    let shader_uri = ResourceLocator::parse(
        "res://shaders/cumulative-event-budget-with-a-long-locator-name.wgsl",
    )
    .unwrap();
    let shader_bytes = approximate_event_bytes(&ResourceEvent {
        kind: ResourceEventKind::Added,
        resource_kind: ResourceKind::Shader,
        id: shader_id,
        locator: Some(shader_uri.clone()),
        previous_locator: None,
        revision: 1,
    });
    let scene_bytes = approximate_event_bytes(&ResourceEvent {
        kind: ResourceEventKind::Added,
        resource_kind: ResourceKind::Scene,
        id: fixture.record.id(),
        locator: Some(fixture.uri.clone()),
        previous_locator: None,
        revision: 1,
    });
    let limits = DynamicSceneAssetReloadLimits {
        max_event_bytes_per_tick: shader_bytes.max(scene_bytes),
        ..DynamicSceneAssetReloadLimits::default()
    };
    let mut queue = DynamicSceneAssetReloadQueue::with_limits(
        fixture.project.clone(),
        events,
        fixture.resources.clone(),
        limits,
    );
    fixture.resources.register_record(ResourceRecord::new(
        shader_id,
        ResourceKind::Shader,
        shader_uri,
    ));
    fixture.register_ready_revision("scene-after-cumulative-filtered-event");

    let first = queue.drain_events(&scheduler);
    assert_eq!(first.raw_events_examined, 1);
    assert_eq!(first.filtered_events, 1);
    assert_eq!(first.events_drained, 0);
    assert_eq!(first.event_bytes_drained, shader_bytes);
    assert!(first.event_budget_exhausted);

    let second = queue.drain_events(&scheduler);
    assert_eq!(second.raw_events_examined, 1);
    assert_eq!(second.events_drained, 1);
    assert_eq!(second.scheduled, 1);

    wait_for_pending(&scheduler, &queue);
    fixture.cleanup();
}

#[test]
fn dynamic_scene_asset_reload_ready_bytes_are_cumulative_within_one_tick() {
    let fixture = SceneReloadFixture::new("asset_reload_cumulative_ready_bytes");
    let second_uri = fixture.copy_main_scene_as("ready-budget-second.scene.toml");
    let mut project = fixture.project.clone();
    project
        .scan_and_import()
        .expect("second scene should import for the ready budget fixture");
    let second_record = project
        .registry()
        .get_by_locator(&second_uri)
        .expect("second scene record should exist")
        .clone();
    let second_scene = match project.load_artifact(&second_uri).unwrap() {
        ImportedAsset::Scene(scene) => scene,
        _ => panic!("expected second scene artifact"),
    };
    let first_bytes = PreparedDynamicSceneSpawn::from_scene_asset(&project, &fixture.scene)
        .unwrap()
        .estimated_bytes();
    let second_bytes = PreparedDynamicSceneSpawn::from_scene_asset(&project, &second_scene)
        .unwrap()
        .estimated_bytes();
    let ready_budget = first_bytes.max(second_bytes);
    let resources = ResourceManager::new();
    let events = Assets::<SceneAsset>::new(resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let limits = DynamicSceneAssetReloadLimits {
        max_active_tasks: 2,
        max_schedules_per_tick: 2,
        max_ready_per_tick: 2,
        max_prepared_scene_bytes: ready_budget,
        max_pending_result_bytes: ready_budget.saturating_mul(2),
        max_ready_bytes_per_tick: ready_budget,
        ..DynamicSceneAssetReloadLimits::default()
    };
    let mut queue = DynamicSceneAssetReloadQueue::with_limits(
        project,
        events,
        fixture.resources.clone(),
        limits,
    );
    resources.register_ready(
        fixture
            .record
            .clone()
            .with_source_hash("ready-budget-first"),
        fixture.scene.clone(),
    );
    resources.register_ready(
        second_record.with_source_hash("ready-budget-second"),
        second_scene,
    );

    let drain = drain_until_events(&mut queue, &scheduler, 2);
    assert_eq!(drain.scheduled, 2);
    wait_for_pending(&scheduler, &queue);

    let first = queue.take_ready_report();
    assert_eq!(first.ready_count(), 1);
    assert!(first.ready[0].result().is_ok());
    assert!(first.collected_bytes <= ready_budget);
    assert_eq!(first.pending_count, 1);
    assert!(first.ready_budget_exhausted);

    let second = queue.take_ready_report();
    assert_eq!(second.ready_count(), 1);
    assert!(second.ready[0].result().is_ok());
    assert!(second.collected_bytes <= ready_budget);
    assert_eq!(second.pending_count, 0);

    fixture.cleanup();
}

#[test]
fn dynamic_scene_asset_reload_target_commits_share_one_cumulative_byte_budget() {
    let fixture = SceneReloadFixture::new("asset_reload_cumulative_target_commit_bytes");
    let second_uri = fixture.copy_main_scene_as("apply-budget-second.scene.toml");
    let mut project = fixture.project.clone();
    project
        .scan_and_import()
        .expect("second scene should import for the apply budget fixture");
    let second_record = project
        .registry()
        .get_by_locator(&second_uri)
        .expect("second scene record should exist")
        .clone();
    let second_scene = match project.load_artifact(&second_uri).unwrap() {
        ImportedAsset::Scene(scene) => scene,
        _ => panic!("expected second scene artifact"),
    };
    let first_prepared =
        PreparedDynamicSceneSpawn::from_scene_asset(&project, &fixture.scene).unwrap();
    let second_prepared =
        PreparedDynamicSceneSpawn::from_scene_asset(&project, &second_scene).unwrap();
    let first_target_bytes = first_prepared
        .capture_world_target(&mut World::empty(), usize::MAX)
        .unwrap()
        .estimated_bytes();
    let second_target_bytes = second_prepared
        .capture_world_target(&mut World::empty(), usize::MAX)
        .unwrap()
        .estimated_bytes();
    let first_reserved = first_prepared
        .estimated_bytes()
        .saturating_add(first_target_bytes);
    let second_reserved = second_prepared
        .estimated_bytes()
        .saturating_add(second_target_bytes);
    let apply_budget = first_reserved.max(second_reserved);
    let resident_budget = first_reserved.saturating_add(second_reserved);
    let ready_budget = first_prepared
        .estimated_bytes()
        .saturating_add(second_prepared.estimated_bytes());
    let resources = ResourceManager::new();
    let events = Assets::<SceneAsset>::new(resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let limits = DynamicSceneAssetReloadLimits {
        max_active_tasks: 2,
        max_schedules_per_tick: 2,
        max_ready_per_tick: 2,
        max_apply_per_tick: 2,
        max_prepared_scene_bytes: first_prepared
            .estimated_bytes()
            .max(second_prepared.estimated_bytes()),
        max_pending_result_bytes: resident_budget,
        max_ready_bytes_per_tick: ready_budget,
        max_apply_bytes_per_tick: apply_budget,
        apply_time_budget: Duration::MAX,
        ..DynamicSceneAssetReloadLimits::default()
    };
    let mut queue =
        DynamicSceneAssetReloadQueue::with_limits(project, events, resources.clone(), limits);
    resources.register_ready(
        fixture
            .record
            .clone()
            .with_source_hash("apply-budget-first"),
        fixture.scene.clone(),
    );
    resources.register_ready(
        second_record.with_source_hash("apply-budget-second"),
        second_scene,
    );

    let drain = drain_until_events(&mut queue, &scheduler, 2);
    assert_eq!(drain.scheduled, 2);
    wait_for_pending(&scheduler, &queue);
    let mut world = World::empty();
    let staged = queue.tick_into(&scheduler, &mut world);
    assert_eq!(staged.applied_count() + staged.failed_count(), 0);
    assert_eq!(queue.target_staging_count(), 2);
    let resident = queue.diagnostics();
    assert!(resident.resident_result_bytes <= resident.max_resident_result_bytes);
    assert!(resident.resident_result_bytes <= limits.max_pending_result_bytes);

    let first_commit = tick_into_until_terminal(&mut queue, &scheduler, &mut world);
    assert_eq!(
        first_commit.applied_count() + first_commit.failed_count(),
        1
    );
    assert!(first_commit.apply.applied_bytes <= apply_budget);
    assert!(first_commit.apply.apply_budget_exhausted);
    assert_eq!(queue.target_staging_count(), 1);

    let second_commit = tick_into_until_terminal(&mut queue, &scheduler, &mut world);
    assert_eq!(
        second_commit.applied_count() + second_commit.failed_count(),
        1
    );
    assert!(second_commit.apply.applied_bytes <= apply_budget);
    assert_eq!(queue.target_staging_count(), 0);

    fixture.cleanup();
}
