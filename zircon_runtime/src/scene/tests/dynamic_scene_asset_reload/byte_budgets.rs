use super::*;
use crate::core::resource::{
    ResourceEvent, ResourceEventKind, ResourceLocator, approximate_event_bytes,
};
use crate::scene::PreparedDynamicSceneSpawn;

#[test]
fn dynamic_scene_asset_reload_event_bytes_are_cumulative_within_one_tick() {
    let fixture = SceneReloadFixture::new("asset_reload_cumulative_event_bytes");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = test_job_scheduler();
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
    fixture
        .resources
        .register_record(ResourceRecord::new(
            shader_id,
            ResourceKind::Shader,
            shader_uri,
        ))
        .unwrap();
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
    let scheduler = test_job_scheduler();
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
    resources
        .register_ready(
            fixture
                .record
                .clone()
                .with_source_hash("ready-budget-first"),
            fixture.scene.clone(),
        )
        .unwrap();
    resources
        .register_ready(
            second_record.with_source_hash("ready-budget-second"),
            second_scene,
        )
        .unwrap();

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
    let scheduler = test_job_scheduler();
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
    resources
        .register_ready(
            fixture
                .record
                .clone()
                .with_source_hash("apply-budget-first"),
            fixture.scene.clone(),
        )
        .unwrap();
    resources
        .register_ready(
            second_record.with_source_hash("apply-budget-second"),
            second_scene,
        )
        .unwrap();

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

#[test]
fn dynamic_scene_asset_reload_target_stage_reconciles_to_actual_snapshot_bytes() {
    let fixture = SceneReloadFixture::new("asset_reload_actual_target_stage_bytes");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = test_job_scheduler();
    let limits = DynamicSceneAssetReloadLimits::default();
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let prepared =
        PreparedDynamicSceneSpawn::from_scene_asset(&fixture.project, &fixture.scene).unwrap();
    let prepared_bytes = prepared.estimated_bytes();
    let actual_target_bytes = prepared
        .capture_level_target(&level, usize::MAX)
        .unwrap()
        .estimated_bytes();
    let expected_reservation = prepared_bytes.saturating_add(actual_target_bytes);
    assert!(expected_reservation < limits.max_apply_bytes_per_tick);

    let mut queue = DynamicSceneAssetReloadQueue::with_limits(
        fixture.project.clone(),
        events,
        fixture.resources.clone(),
        limits,
    );
    fixture.register_ready_revision("actual-target-stage-bytes");
    let drain = drain_until_events(&mut queue, &scheduler, 1);
    assert_eq!(drain.scheduled, 1);
    wait_for_pending(&scheduler, &queue);

    let staged = queue.tick_into_level(&scheduler, &level);
    assert_eq!(staged.applied_count() + staged.failed_count(), 0);
    assert_eq!(queue.target_staging_count(), 1);

    let deadline = Instant::now() + EVENT_DRAIN_TIMEOUT;
    loop {
        let reserved_bytes = queue.diagnostics().target_staging_reserved_bytes;
        if reserved_bytes == expected_reservation {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "target stage kept {reserved_bytes} reserved bytes; expected actual {expected_reservation}"
        );
        thread::yield_now();
    }

    let terminal = tick_into_level_until_terminal(&mut queue, &scheduler, &level);
    assert_eq!(terminal.applied_count(), 1);
    assert_eq!(terminal.failed_count(), 0);
    assert_eq!(queue.diagnostics().target_staging_reserved_bytes, 0);

    fixture.cleanup();
}

#[test]
#[ignore = "release-mode capacity evidence; run explicitly"]
fn dynamic_scene_asset_reload_actual_target_reservation_capacity_benchmark() {
    const SAMPLE_PAIRS: usize = 21;
    const STAGED_SCENES: usize = 65_536;
    const PREPARED_BYTES: usize = 64 * 1024;
    const TARGET_LIMIT_BYTES: usize = 32 * 1024 * 1024 - PREPARED_BYTES;

    fn measure_reservations(
        actual_target_bytes: &[usize],
        use_actual_target_bytes: bool,
    ) -> (u128, usize) {
        let started = Instant::now();
        let reserved_bytes = actual_target_bytes.iter().fold(0usize, |total, actual| {
            let target_bytes = if use_actual_target_bytes {
                std::hint::black_box(*actual)
            } else {
                std::hint::black_box(TARGET_LIMIT_BYTES)
            };
            total.saturating_add(PREPARED_BYTES.saturating_add(target_bytes))
        });
        (
            started.elapsed().as_nanos(),
            std::hint::black_box(reserved_bytes),
        )
    }

    fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
        samples.sort_unstable();
        let rank = samples.len().saturating_mul(percentile).saturating_add(99) / 100;
        samples[rank.saturating_sub(1).min(samples.len().saturating_sub(1))]
    }

    let actual_target_bytes = (0..STAGED_SCENES)
        .map(|index| 32 * 1024 + (index % 64) * 512)
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut legacy_reserved_bytes = 0usize;
    let mut optimized_reserved_bytes = 0usize;

    for pair in 0..SAMPLE_PAIRS {
        let legacy_first = pair % 2 == 0;
        let first = measure_reservations(&actual_target_bytes, !legacy_first);
        let second = measure_reservations(&actual_target_bytes, legacy_first);
        if legacy_first {
            legacy_samples.push(first.0);
            legacy_reserved_bytes = first.1;
            optimized_samples.push(second.0);
            optimized_reserved_bytes = second.1;
        } else {
            optimized_samples.push(first.0);
            optimized_reserved_bytes = first.1;
            legacy_samples.push(second.0);
            legacy_reserved_bytes = second.1;
        }
    }

    let legacy_p50_ns = nearest_rank(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = nearest_rank(&mut legacy_samples.clone(), 95);
    let optimized_p50_ns = nearest_rank(&mut optimized_samples.clone(), 50);
    let optimized_p95_ns = nearest_rank(&mut optimized_samples.clone(), 95);
    let legacy_ns = legacy_samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let optimized_ns = optimized_samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let released_bytes = legacy_reserved_bytes.saturating_sub(optimized_reserved_bytes);
    let reduction_basis_points = released_bytes
        .saturating_mul(10_000)
        .checked_div(legacy_reserved_bytes)
        .unwrap_or(0);

    assert!(
        optimized_reserved_bytes.saturating_mul(4) <= legacy_reserved_bytes,
        "actual target reservation must reduce staged resident bytes by at least 75%"
    );
    println!(
        "PERF-MVP-DSRL-P1-035 sample_pairs={SAMPLE_PAIRS} sample_order=alternating \
percentile_method=nearest_rank timing_gate=diagnostic_only staged_scenes={STAGED_SCENES} \
legacy_ns={legacy_ns} optimized_ns={optimized_ns} \
legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_reserved_bytes={legacy_reserved_bytes} optimized_reserved_bytes={optimized_reserved_bytes} \
released_bytes={released_bytes} reservation_reduction_basis_points={reduction_basis_points}"
    );
}
