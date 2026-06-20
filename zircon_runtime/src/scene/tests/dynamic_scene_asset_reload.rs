use std::{
    fs,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use crate::{
    asset::{AssetEventKind, AssetUri, Assets, ImportedAsset, ProjectManager, SceneAsset},
    core::{
        resource::{ResourceDiagnostic, ResourceManager, ResourceRecord},
        JobScheduler,
    },
    scene::{
        DefaultLevelManager, DynamicSceneAssetReloadDrainReport, DynamicSceneAssetReloadQueue,
        DynamicSceneAssetReloadSkipReason, World,
    },
};

use super::support::{create_test_project, unique_temp_project_root};

const EVENT_DRAIN_TIMEOUT: Duration = Duration::from_secs(1);
const MAIN_SCENE_URI: &str = "res://scenes/main.scene.toml";

#[test]
fn dynamic_scene_asset_reload_supersedes_older_pending_scene_revision() {
    let fixture = SceneReloadFixture::new("asset_reload_supersedes_pending");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let mut queue = DynamicSceneAssetReloadQueue::new(fixture.project.clone(), events);

    fixture.register_ready_revision("scene-v1");
    fixture.register_ready_revision("scene-v2");

    let drain = drain_until_events(&mut queue, &scheduler, 2);
    assert_eq!(drain.events_drained, 2);
    assert_eq!(drain.scheduled, 2);
    assert_eq!(drain.skipped_count(), 0);
    assert_eq!(drain.superseded_pending_count(), 1);
    assert_eq!(drain.superseded_pending[0].event().revision(), 1);
    assert_eq!(drain.superseded_pending[0].latest_revision(), 2);
    assert_eq!(queue.pending_count(), 1);

    let pending = queue.pending_report();
    assert_eq!(pending.pending_count(), 1);
    assert_eq!(
        pending.pending[0].event().event_kind(),
        AssetEventKind::Modified
    );
    assert_eq!(pending.pending[0].event().revision(), 2);

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
fn dynamic_scene_asset_reload_skips_removed_and_reload_failed_events() {
    let fixture = SceneReloadFixture::new("asset_reload_skips_terminal_events");
    fixture.register_ready_revision("scene-v1");
    fixture
        .resources
        .start_reload(fixture.record.id(), Vec::new())
        .expect("scene asset should enter reload state");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let mut queue = DynamicSceneAssetReloadQueue::new(fixture.project.clone(), events);

    fixture.resources.fail_reload(
        fixture.record.id(),
        vec![ResourceDiagnostic::error("scene reload failed")],
    );
    fixture.resources.remove_by_locator(&fixture.uri);

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

    let ready = queue.take_ready_report();
    assert_eq!(ready.ready_count(), 0);
    assert_eq!(ready.stale_count(), 0);
    assert_eq!(ready.pending_count, 0);

    fixture.cleanup();
}

#[test]
fn dynamic_scene_asset_reload_tick_into_applies_ready_payload_to_world() {
    let fixture = SceneReloadFixture::new("asset_reload_tick_into_world");
    let events = Assets::<SceneAsset>::new(fixture.resources.clone()).subscribe_events();
    let scheduler = JobScheduler::default();
    let mut queue = DynamicSceneAssetReloadQueue::new(fixture.project.clone(), events);

    fixture.register_ready_revision("scene-frame-world");

    let drain = drain_until_events(&mut queue, &scheduler, 1);
    assert_eq!(drain.events_drained, 1);
    assert_eq!(drain.scheduled, 1);
    assert_eq!(queue.pending_count(), 1);

    wait_for_pending(&scheduler, &queue);
    let mut world = World::empty();
    let frame = queue.tick_into(&scheduler, &mut world);

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
    let mut queue = DynamicSceneAssetReloadQueue::new(fixture.project.clone(), events);

    fixture.register_ready_revision("scene-frame-level");

    let drain = drain_until_events(&mut queue, &scheduler, 1);
    assert_eq!(drain.events_drained, 1);
    assert_eq!(drain.scheduled, 1);
    assert_eq!(queue.pending_count(), 1);

    wait_for_pending(&scheduler, &queue);
    let level = DefaultLevelManager::default().create_level(World::empty(), Default::default());
    let frame = queue.tick_into_level(&scheduler, &level);

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
    let mut queue = DynamicSceneAssetReloadQueue::new(project, events);

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
    let frame = queue.tick_into(&scheduler, &mut world);

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
        .pending()
        .iter()
        .map(|task| task.task().completion_handle())
        .collect::<Vec<_>>();
    scheduler.wait_all(&handles);
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
        self.resources.register_ready(
            self.record.clone().with_source_hash(source_hash),
            self.scene.clone(),
        );
    }

    fn copy_main_scene_as(&self, file_name: &str) -> AssetUri {
        let scenes_root = self.project.paths().assets_root().join("scenes");
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
