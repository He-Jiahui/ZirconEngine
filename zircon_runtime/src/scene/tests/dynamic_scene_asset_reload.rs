use std::{
    fs,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use crate::{
    asset::{AssetEventKind, AssetUri, Assets, ImportedAsset, ProjectManager, SceneAsset},
    core::{
        JobScheduler, TaskPool, TaskPoolDescriptor,
        framework::tasks::TaskCancellationPolicy,
        resource::{ResourceDiagnostic, ResourceId, ResourceKind, ResourceManager, ResourceRecord},
    },
    scene::{
        DefaultLevelManager, DynamicSceneAssetReloadDrainReport, DynamicSceneAssetReloadLimits,
        DynamicSceneAssetReloadQueue, DynamicSceneAssetReloadSkipReason, World,
    },
};

use super::support::{create_test_project, unique_temp_project_root};

const EVENT_DRAIN_TIMEOUT: Duration = Duration::from_secs(1);
const MAIN_SCENE_URI: &str = "res://scenes/main.scene.toml";

#[path = "dynamic_scene_asset_reload/byte_budgets.rs"]
mod byte_budgets;

#[test]
fn dynamic_scene_asset_reload_supersedes_older_pending_scene_revision() {
    let fixture = SceneReloadFixture::new("asset_reload_supersedes_pending");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let mut queue = DynamicSceneAssetReloadQueue::new(
        fixture.project.clone(),
        events,
        fixture.resources.clone(),
    );

    fixture.register_ready_revision("scene-v1");
    let first = drain_until_events(&mut queue, &scheduler, 1);
    assert_eq!(first.scheduled, 1);

    fixture.register_ready_revision("scene-v2");
    let drain = drain_until_events(&mut queue, &scheduler, 1);
    assert_eq!(drain.events_drained, 1);
    assert!(drain.scheduled <= 1);
    assert_eq!(drain.skipped_count(), 0);
    assert_eq!(drain.superseded_pending_count(), 1);
    assert_eq!(drain.superseded_pending[0].event().revision(), 1);
    assert_eq!(drain.superseded_pending[0].latest_revision(), 2);
    assert_eq!(
        drain.superseded_pending[0].cancellation_requested(),
        matches!(
            drain.superseded_pending[0].previous_state(),
            crate::core::framework::tasks::AsyncTaskState::Pending
                | crate::core::framework::tasks::AsyncTaskState::Running
        )
    );
    assert_eq!(queue.pending_count(), 1);

    wait_for_pending(&scheduler, &queue);
    let continuation = queue.drain_events(&scheduler);
    assert_eq!(
        first.scheduled + drain.scheduled + continuation.scheduled,
        2
    );
    let pending = queue.pending_report();
    assert_eq!(pending.pending_count(), 1);
    assert_eq!(
        pending.pending[0].event().event_kind(),
        AssetEventKind::Modified
    );
    assert_eq!(pending.pending[0].event().revision(), 2);
    assert_eq!(
        pending.pending[0].descriptor().cancellation_policy,
        TaskCancellationPolicy::CancelOnDrop
    );

    wait_for_pending(&scheduler, &queue);
    let ready = queue.take_ready_report();
    assert_eq!(ready.ready_count(), 1);
    assert_eq!(ready.stale_count(), 0);
    assert_eq!(ready.pending_count, 0);
    assert_eq!(ready.ready[0].event().revision(), 2);

    let mut world = World::empty();
    let apply = ready.spawn_ready_into(&mut world);
    assert_eq!(apply.applied_count(), 1);
    assert_eq!(apply.failed_count(), 0);
    assert_eq!(apply.stale_count(), 0);
    assert_eq!(apply.applied[0].entity_count(), 2);
    assert_eq!(world.node_records().len(), 2);

    fixture.cleanup();
}

#[test]
fn dynamic_scene_asset_reload_keeps_one_physical_worker_across_slow_prepare_matrix() {
    for prepare_delay in [
        Duration::ZERO,
        Duration::from_millis(10),
        Duration::from_secs(1),
    ] {
        let fixture_name = format!(
            "asset_reload_three_rapid_revisions_{}ms",
            prepare_delay.as_millis()
        );
        let fixture = SceneReloadFixture::new(&fixture_name);
        let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
        let scheduler = JobScheduler::from_pool(TaskPool::new(
            TaskPoolDescriptor::compute().with_worker_threads(1),
        ));
        let (blocker_started_tx, blocker_started_rx) = std::sync::mpsc::sync_channel(0);
        let (release_blocker_tx, release_blocker_rx) = std::sync::mpsc::sync_channel(0);
        scheduler.spawn(move || {
            blocker_started_tx.send(()).unwrap();
            release_blocker_rx.recv().unwrap();
            thread::sleep(prepare_delay);
        });
        blocker_started_rx.recv().unwrap();
        let mut queue = DynamicSceneAssetReloadQueue::new(
            fixture.project.clone(),
            events,
            fixture.resources.clone(),
        );

        fixture.register_ready_revision("scene-v1");
        assert_eq!(drain_until_events(&mut queue, &scheduler, 1).scheduled, 1);
        fixture.register_ready_revision("scene-v2");
        assert_eq!(drain_until_events(&mut queue, &scheduler, 1).scheduled, 0);
        fixture.register_ready_revision("scene-v3");
        let third = drain_until_events(&mut queue, &scheduler, 1);

        assert_eq!(third.scheduled, 0, "delay={prepare_delay:?}");
        assert_eq!(queue.pending_tasks().len(), 1, "delay={prepare_delay:?}");
        assert_eq!(queue.pending_count(), 1, "delay={prepare_delay:?}");
        assert_eq!(
            queue.diagnostics().max_active_tasks,
            1,
            "delay={prepare_delay:?}"
        );

        release_blocker_tx.send(()).unwrap();
        wait_for_pending(&scheduler, &queue);
        let continuation = queue.drain_events(&scheduler);
        assert_eq!(continuation.scheduled, 1, "delay={prepare_delay:?}");
        assert_eq!(
            queue.pending_report().pending[0].event().revision(),
            3,
            "delay={prepare_delay:?}"
        );

        fixture.cleanup();
    }
}

#[test]
fn dynamic_scene_asset_reload_asset_scale_matrix_honors_single_event_and_task_budget() {
    for asset_count in [1usize, 1_000, 100_000] {
        let fixture = SceneReloadFixture::new(&format!("asset_reload_scale_{asset_count}"));
        let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
        let scheduler = JobScheduler::default();
        let limits = DynamicSceneAssetReloadLimits {
            max_events_per_tick: 1,
            max_schedules_per_tick: 1,
            max_active_tasks: 1,
            ..DynamicSceneAssetReloadLimits::default()
        };
        let mut queue = DynamicSceneAssetReloadQueue::with_limits(
            fixture.project.clone(),
            events,
            fixture.resources.clone(),
            limits,
        );

        for index in 0..asset_count {
            fixture
                .resources
                .register_record(ResourceRecord::new(
                    ResourceId::from_stable_label(&format!("scale-scene-{asset_count}-{index}")),
                    ResourceKind::Scene,
                    AssetUri::parse(&format!(
                        "res://scenes/scale-{asset_count}-{index}.scene.toml"
                    ))
                    .expect("scale fixture URI should be valid"),
                ))
                .unwrap();
        }

        let drain = queue.drain_events(&scheduler);
        let diagnostics = queue.diagnostics();
        assert!(drain.raw_events_examined <= 1, "assets={asset_count}");
        assert!(drain.events_drained <= 1, "assets={asset_count}");
        assert!(drain.scheduled <= 1, "assets={asset_count}");
        assert!(queue.pending_count() <= 1, "assets={asset_count}");
        assert!(diagnostics.active_tasks <= 1, "assets={asset_count}");
        assert!(diagnostics.max_active_tasks <= 1, "assets={asset_count}");

        fixture.cleanup();
    }
}

#[test]
fn dynamic_scene_asset_reload_resumes_event_drain_at_the_frame_budget() {
    let fixture = SceneReloadFixture::new("asset_reload_event_budget");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let limits = DynamicSceneAssetReloadLimits {
        max_events_per_tick: 1,
        max_schedules_per_tick: 1,
        ..DynamicSceneAssetReloadLimits::default()
    };
    let mut queue = DynamicSceneAssetReloadQueue::with_limits(
        fixture.project.clone(),
        events,
        fixture.resources.clone(),
        limits,
    );

    fixture.register_ready_revision("scene-budget-v1");
    let first = queue.drain_events(&scheduler);
    assert_eq!(first.events_drained, 1);
    assert_eq!(first.scheduled, 1);
    assert!(first.event_budget_exhausted);
    assert_eq!(queue.pending_count(), 1);

    fixture.register_ready_revision("scene-budget-v2");
    let second = queue.drain_events(&scheduler);
    assert_eq!(second.events_drained, 1);
    assert!(second.scheduled <= 1);
    assert_eq!(second.superseded_pending_count(), 1);
    let expected_cancellation_requests =
        usize::from(second.superseded_pending[0].cancellation_requested());
    assert_eq!(queue.pending_count(), 1);

    wait_for_pending(&scheduler, &queue);
    let continuation = queue.drain_events(&scheduler);
    assert_eq!(
        first.scheduled + second.scheduled + continuation.scheduled,
        2
    );
    let diagnostics = queue.diagnostics();
    assert_eq!(diagnostics.events_drained, 2);
    assert_eq!(diagnostics.tasks_scheduled, 2);
    assert_eq!(
        diagnostics.cancellation_requests,
        expected_cancellation_requests as u64
    );
    assert_eq!(diagnostics.active_tasks, 1);
    assert_eq!(diagnostics.max_active_tasks, 1);

    fixture.cleanup();
}

#[test]
fn dynamic_scene_asset_reload_budgets_filtered_raw_events() {
    let fixture = SceneReloadFixture::new("asset_reload_filtered_event_budget");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let limits = DynamicSceneAssetReloadLimits {
        max_events_per_tick: 1,
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
            ResourceId::from_stable_label("unrelated shader event"),
            ResourceKind::Shader,
            AssetUri::parse("res://shaders/unrelated.wgsl").unwrap(),
        ))
        .unwrap();
    fixture.register_ready_revision("scene-after-filtered-event");

    let first = queue.drain_events(&scheduler);
    assert_eq!(first.raw_events_examined, 1);
    assert_eq!(first.filtered_events, 1);
    assert_eq!(first.events_drained, 0);
    assert_eq!(first.scheduled, 0);
    assert!(first.event_budget_exhausted);
    assert!(
        queue.has_pending_work(),
        "receiver backlog must keep reactive frame demand immediate"
    );

    let second = queue.drain_events(&scheduler);
    assert_eq!(second.raw_events_examined, 1);
    assert_eq!(second.filtered_events, 0);
    assert_eq!(second.events_drained, 1);
    assert_eq!(second.scheduled, 1);
    assert!(second.event_bytes_drained > 0);

    fixture.cleanup();
}

#[test]
fn dynamic_scene_asset_reload_reports_resource_event_generation_gaps() {
    let fixture = SceneReloadFixture::new("asset_reload_generation_gap");
    fixture.register_ready_revision("scene-gap-current");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let mut queue = DynamicSceneAssetReloadQueue::with_limits(
        fixture.project.clone(),
        events,
        fixture.resources.clone(),
        DynamicSceneAssetReloadLimits::default(),
    );
    for index in 0..4_128 {
        fixture
            .resources
            .register_record(ResourceRecord::new(
                ResourceId::from_stable_label(&format!("gap-shader-{index}")),
                ResourceKind::Shader,
                AssetUri::parse(&format!("res://shaders/gap-{index}.wgsl")).unwrap(),
            ))
            .unwrap();
    }

    let drain = queue.drain_events(&scheduler);

    assert!(drain.generation_gap.is_some());
    assert_eq!(drain.events_drained, 0);
    assert_eq!(drain.scheduled, 0);
    assert_eq!(queue.diagnostics().generation_gaps, 1);

    let reconciled = queue.drain_events(&scheduler);
    assert_eq!(reconciled.events_drained, 1);
    assert_eq!(reconciled.scheduled, 1);
    assert_eq!(queue.pending_report().pending[0].event().revision(), 1);

    fixture.cleanup();
}

#[test]
fn dynamic_scene_asset_reload_reconciliation_is_incremental_and_skips_pending_rows() {
    let fixture = SceneReloadFixture::new("asset_reload_incremental_reconciliation");
    fixture.register_ready_revision("scene-reconcile-ready");
    fixture
        .resources
        .register_record(ResourceRecord::new(
            ResourceId::from_stable_label("pending reconciliation scene"),
            ResourceKind::Scene,
            AssetUri::parse("res://scenes/pending.scene.toml").unwrap(),
        ))
        .unwrap();
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let limits = DynamicSceneAssetReloadLimits {
        max_events_per_tick: 1,
        max_schedules_per_tick: 1,
        ..DynamicSceneAssetReloadLimits::default()
    };
    let mut queue = DynamicSceneAssetReloadQueue::with_limits(
        fixture.project.clone(),
        events,
        fixture.resources.clone(),
        limits,
    );
    publish_gap_fixture_events(&fixture.resources);

    assert!(queue.drain_events(&scheduler).generation_gap.is_some());
    let ready = queue.drain_events(&scheduler);
    assert_eq!(ready.raw_events_examined, 1);
    assert_eq!(ready.events_drained, 1);
    assert_eq!(ready.scheduled, 1);
    assert!(ready.event_budget_exhausted);

    let pending = queue.drain_events(&scheduler);
    assert_eq!(pending.raw_events_examined, 1);
    assert_eq!(pending.filtered_events, 1);
    assert_eq!(pending.events_drained, 0);
    assert_eq!(pending.scheduled, 0);

    wait_for_pending(&scheduler, &queue);
    fixture.cleanup();
}

#[test]
fn dynamic_scene_asset_reload_reconciliation_obeys_event_byte_budget() {
    let fixture = SceneReloadFixture::new("asset_reload_reconciliation_byte_budget");
    fixture.register_ready_revision("scene-reconcile-byte-budget");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let limits = DynamicSceneAssetReloadLimits {
        max_event_bytes_per_tick: 1,
        ..DynamicSceneAssetReloadLimits::default()
    };
    let mut queue = DynamicSceneAssetReloadQueue::with_limits(
        fixture.project.clone(),
        events,
        fixture.resources.clone(),
        limits,
    );
    publish_gap_fixture_events(&fixture.resources);

    assert!(queue.drain_events(&scheduler).generation_gap.is_some());
    let reconciled = queue.drain_events(&scheduler);

    assert_eq!(reconciled.raw_events_examined, 1);
    assert_eq!(reconciled.events_drained, 0);
    assert_eq!(reconciled.scheduled, 0);
    assert!(reconciled.event_budget_exhausted);
    assert_eq!(
        reconciled.skipped_count_for(DynamicSceneAssetReloadSkipReason::CapacityExceeded),
        1
    );

    fixture.cleanup();
}

#[test]
fn dynamic_scene_asset_reload_skips_removed_and_reload_failed_events() {
    let fixture = SceneReloadFixture::new("asset_reload_skips_terminal_events");
    fixture.register_ready_revision("scene-v1");
    fixture
        .resources
        .start_reload(fixture.record.id(), Vec::new())
        .expect("scene asset should enter reload state");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let mut queue = DynamicSceneAssetReloadQueue::new(
        fixture.project.clone(),
        events,
        fixture.resources.clone(),
    );

    fixture
        .resources
        .fail_reload(
            fixture.record.id(),
            vec![ResourceDiagnostic::error("scene reload failed")],
        )
        .unwrap();
    fixture.resources.remove_by_locator(&fixture.uri).unwrap();

    let drain = drain_until_events(&mut queue, &scheduler, 2);
    assert_eq!(drain.events_drained, 2);
    assert_eq!(drain.scheduled, 0);
    assert_eq!(drain.skipped_count(), 2);
    assert_eq!(
        drain.skipped_count_for(DynamicSceneAssetReloadSkipReason::ReloadFailed),
        1
    );
    assert_eq!(
        drain.skipped_count_for(DynamicSceneAssetReloadSkipReason::Removed),
        1
    );
    assert_eq!(drain.pending_count, 0);
    assert_eq!(queue.pending_count(), 0);
    assert_eq!(queue.diagnostics().latest_entries, 0);

    let ready = queue.take_ready_report();
    assert_eq!(ready.ready_count(), 0);
    assert_eq!(ready.stale_count(), 0);
    assert_eq!(ready.pending_count, 0);

    fixture.cleanup();
}

#[test]
fn dynamic_scene_asset_reload_removed_asset_can_recreate_same_id_from_revision_one() {
    let fixture = SceneReloadFixture::new("asset_reload_recreates_removed_id");
    fixture.register_ready_revision("scene-before-remove");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let mut queue = DynamicSceneAssetReloadQueue::new(
        fixture.project.clone(),
        events,
        fixture.resources.clone(),
    );

    fixture.resources.remove_by_locator(&fixture.uri).unwrap();
    let removed = drain_until_events(&mut queue, &scheduler, 1);
    assert_eq!(
        removed.skipped_count_for(DynamicSceneAssetReloadSkipReason::Removed),
        1
    );
    assert_eq!(queue.diagnostics().latest_entries, 0);

    fixture.register_ready_revision("scene-recreated-revision-one");
    let recreated = drain_until_events(&mut queue, &scheduler, 1);
    assert_eq!(recreated.scheduled, 1);
    assert_eq!(
        recreated.skipped_count_for(DynamicSceneAssetReloadSkipReason::StaleRevision),
        0
    );
    wait_for_pending(&scheduler, &queue);
    let ready = queue.take_ready_report();
    assert_eq!(ready.ready_count(), 1);
    assert_eq!(ready.stale_count(), 0);

    fixture.cleanup();
}

#[test]
fn dynamic_scene_asset_reload_tick_into_applies_ready_payload_to_world() {
    let fixture = SceneReloadFixture::new("asset_reload_tick_into_world");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let mut queue = DynamicSceneAssetReloadQueue::new(
        fixture.project.clone(),
        events,
        fixture.resources.clone(),
    );

    fixture.register_ready_revision("scene-frame-world");

    let drain = drain_until_events(&mut queue, &scheduler, 1);
    assert_eq!(drain.events_drained, 1);
    assert_eq!(drain.scheduled, 1);
    assert_eq!(queue.pending_count(), 1);

    wait_for_pending(&scheduler, &queue);
    let mut world = World::empty();
    let staged = queue.tick_into(&scheduler, &mut world);

    assert_eq!(staged.applied_count(), 0);
    assert_eq!(staged.failed_count(), 0);
    assert_eq!(queue.target_staging_count(), 1);
    assert!(world.node_records().is_empty());
    let diagnostics = queue.diagnostics();
    assert_eq!(diagnostics.target_staging_tasks, 1);
    assert_eq!(diagnostics.max_active_tasks, 1);
    assert!(diagnostics.target_staging_reserved_bytes <= queue.limits().max_pending_result_bytes);
    assert_eq!(
        diagnostics.resident_result_bytes,
        diagnostics
            .ready_result_bytes
            .saturating_add(diagnostics.target_staging_reserved_bytes)
    );
    assert!(diagnostics.resident_result_bytes <= queue.limits().max_pending_result_bytes);
    assert!(diagnostics.max_target_capture_duration >= staged.apply.target_capture_elapsed);

    let frame = tick_into_until_terminal(&mut queue, &scheduler, &mut world);

    assert_eq!(frame.events_drained(), 0);
    assert_eq!(frame.scheduled_count(), 0);
    assert_eq!(frame.applied_count(), 1);
    assert_eq!(frame.failed_count(), 0);
    assert_eq!(frame.stale_count(), 0);
    assert_eq!(frame.pending_count(), 0);
    assert_eq!(frame.apply.applied[0].event().revision(), 1);
    assert_eq!(frame.apply.applied[0].entity_count(), 2);
    assert_eq!(world.node_records().len(), 2);

    fixture.cleanup();
}

#[test]
fn dynamic_scene_asset_reload_tick_into_level_applies_ready_payload_to_level_world() {
    let fixture = SceneReloadFixture::new("asset_reload_tick_into_level");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let mut queue = DynamicSceneAssetReloadQueue::new(
        fixture.project.clone(),
        events,
        fixture.resources.clone(),
    );

    fixture.register_ready_revision("scene-frame-level");

    let drain = drain_until_events(&mut queue, &scheduler, 1);
    assert_eq!(drain.events_drained, 1);
    assert_eq!(drain.scheduled, 1);
    assert_eq!(queue.pending_count(), 1);

    wait_for_pending(&scheduler, &queue);
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let staged = queue.tick_into_level(&scheduler, &level);

    assert_eq!(staged.applied_count(), 0);
    assert_eq!(staged.failed_count(), 0);
    assert_eq!(queue.target_staging_count(), 1);
    assert_eq!(staged.apply.target_capture_elapsed, Duration::ZERO);
    assert!(level.with_world(|world| world.node_records().is_empty()));

    let frame = tick_into_level_until_terminal(&mut queue, &scheduler, &level);

    assert_eq!(frame.events_drained(), 0);
    assert_eq!(frame.scheduled_count(), 0);
    assert_eq!(frame.applied_count(), 1);
    assert_eq!(frame.failed_count(), 0);
    assert_eq!(frame.stale_count(), 0);
    assert_eq!(frame.pending_count(), 0);
    assert_eq!(frame.apply.applied[0].event().revision(), 1);
    assert_eq!(frame.apply.applied[0].entity_count(), 2);
    assert_eq!(level.with_world(|world| world.node_records().len()), 2);

    fixture.cleanup();
}

#[test]
fn dynamic_scene_asset_reload_renamed_scene_event_schedules_new_project_uri() {
    let fixture = SceneReloadFixture::new("asset_reload_renamed_scene_uri");
    let renamed_uri = fixture.copy_main_scene_as("renamed.scene.toml");
    let mut project = fixture.project.clone();
    project
        .scan_and_import()
        .expect("renamed scene should import into the project registry");
    fixture.register_ready_revision("scene-before-rename");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let mut queue = DynamicSceneAssetReloadQueue::new(project, events, fixture.resources.clone());

    fixture
        .resources
        .rename(&fixture.uri, renamed_uri.clone())
        .expect("scene resource should rename");

    let drain = drain_until_events(&mut queue, &scheduler, 1);
    assert_eq!(drain.events_drained, 1);
    assert_eq!(drain.scheduled, 1);
    assert_eq!(drain.skipped_count(), 0);
    assert_eq!(queue.pending_count(), 1);

    let pending = queue.pending_report();
    assert_eq!(pending.pending_count(), 1);
    assert_eq!(
        pending.pending[0].event().event_kind(),
        AssetEventKind::Renamed
    );
    assert_eq!(pending.pending[0].event().locator(), Some(&renamed_uri));
    assert_eq!(
        pending.pending[0].event().previous_locator(),
        Some(&fixture.uri)
    );

    wait_for_pending(&scheduler, &queue);
    let mut world = World::empty();
    let staged = queue.tick_into(&scheduler, &mut world);

    assert_eq!(staged.applied_count(), 0);
    assert_eq!(queue.target_staging_count(), 1);
    assert!(world.node_records().is_empty());

    let frame = tick_into_until_terminal(&mut queue, &scheduler, &mut world);

    assert_eq!(frame.applied_count(), 1);
    assert_eq!(frame.failed_count(), 0);
    assert_eq!(frame.pending_count(), 0);
    assert_eq!(
        frame.apply.applied[0].event().event_kind(),
        AssetEventKind::Renamed
    );
    assert_eq!(frame.apply.applied[0].event().locator(), Some(&renamed_uri));
    assert_eq!(world.node_records().len(), 2);

    fixture.cleanup();
}

fn drain_until_events(
    queue: &mut DynamicSceneAssetReloadQueue,
    scheduler: &JobScheduler,
    expected_events: usize,
) -> DynamicSceneAssetReloadDrainReport {
    let deadline = Instant::now() + EVENT_DRAIN_TIMEOUT;
    let mut total = DynamicSceneAssetReloadDrainReport::default();

    loop {
        let drain = queue.drain_events(scheduler);
        total.events_drained += drain.events_drained;
        total.scheduled += drain.scheduled;
        total.skipped.extend(drain.skipped);
        total.superseded_pending.extend(drain.superseded_pending);
        total.receiver_disconnected |= drain.receiver_disconnected;
        total.pending_count = drain.pending_count;

        if total.events_drained >= expected_events || Instant::now() >= deadline {
            return total;
        }

        thread::sleep(Duration::from_millis(1));
    }
}

fn wait_for_pending(scheduler: &JobScheduler, queue: &DynamicSceneAssetReloadQueue) {
    let handles = queue
        .pending_tasks()
        .map(|task| task.task().completion_handle())
        .collect::<Vec<_>>();
    scheduler.wait_all(&handles);
}

fn tick_into_until_terminal(
    queue: &mut DynamicSceneAssetReloadQueue,
    scheduler: &JobScheduler,
    world: &mut World,
) -> crate::scene::DynamicSceneAssetReloadFrameApplyReport {
    let deadline = Instant::now() + EVENT_DRAIN_TIMEOUT;
    loop {
        let frame = queue.tick_into(scheduler, world);
        if frame.applied_count() + frame.failed_count() > 0 || Instant::now() >= deadline {
            return frame;
        }
        thread::yield_now();
    }
}

fn tick_into_level_until_terminal(
    queue: &mut DynamicSceneAssetReloadQueue,
    scheduler: &JobScheduler,
    level: &crate::scene::LevelSystem,
) -> crate::scene::DynamicSceneAssetReloadFrameApplyReport {
    let deadline = Instant::now() + EVENT_DRAIN_TIMEOUT;
    loop {
        let frame = queue.tick_into_level(scheduler, level);
        if frame.applied_count() + frame.failed_count() > 0 || Instant::now() >= deadline {
            return frame;
        }
        thread::yield_now();
    }
}

fn publish_gap_fixture_events(resources: &ResourceManager) {
    for index in 0..4_128 {
        resources
            .register_record(ResourceRecord::new(
                ResourceId::from_stable_label(&format!("reconciliation-gap-shader-{index}")),
                ResourceKind::Shader,
                AssetUri::parse(&format!("res://shaders/reconciliation-gap-{index}.wgsl")).unwrap(),
            ))
            .unwrap();
    }
}

struct SceneReloadFixture {
    root: PathBuf,
    project: ProjectManager,
    resources: ResourceManager,
    uri: AssetUri,
    record: ResourceRecord,
    scene: SceneAsset,
}

impl SceneReloadFixture {
    fn new(label: &str) -> Self {
        let root = unique_temp_project_root(label);
        let project = create_test_project(&root);
        let resources = ResourceManager::new();
        let uri = AssetUri::parse(MAIN_SCENE_URI).expect("main scene uri should parse");
        let record = project
            .registry()
            .get_by_locator(&uri)
            .expect("main scene record should exist")
            .clone();
        let scene = match project
            .load_artifact(&uri)
            .expect("main scene artifact should load")
        {
            ImportedAsset::Scene(scene) => scene,
            _ => panic!("expected scene artifact for {uri}"),
        };

        Self {
            root,
            project,
            resources,
            uri,
            record,
            scene,
        }
    }

    fn register_ready_revision(&self, source_hash: &str) {
        self.resources
            .register_ready(
                self.record.clone().with_source_hash(source_hash),
                self.scene.clone(),
            )
            .unwrap();
    }

    fn copy_main_scene_as(&self, file_name: &str) -> AssetUri {
        let scenes_root = self
            .project
            .primary_project_asset_root()
            .unwrap()
            .join("scenes");
        fs::copy(
            scenes_root.join("main.scene.toml"),
            scenes_root.join(file_name),
        )
        .expect("copy test scene asset");
        AssetUri::parse(&format!("res://scenes/{file_name}"))
            .expect("renamed scene uri should parse")
    }

    fn cleanup(self) {
        let _ = fs::remove_dir_all(self.root);
    }
}
