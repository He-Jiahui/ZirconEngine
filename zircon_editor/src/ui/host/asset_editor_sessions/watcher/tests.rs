use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::budget::{UiAssetWatchBudget, UiAssetWatchBudgetError};
use super::ingress::UiAssetWatchIngressHandle;
use super::service::{UiAssetWatchPollStart, UiAssetWatchReconcileCursor, UiAssetWorkspaceWatcher};
use super::UiAssetWorkspaceWatchPollReport;
use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession};
use crate::ui::host::asset_editor_sessions::refresh::reconcile::collect_ui_asset_reconcile_batch;
use crate::ui::host::asset_editor_sessions::UiAssetWorkspaceEntry;
use crate::ui::workbench::view::ViewInstanceId;
use zircon_runtime_interface::ui::layout::UiSize;
use zircon_runtime_interface::ui::template::UiAssetKind;

fn root() -> PathBuf {
    PathBuf::from(r"C:\zircon-project\assets")
}

fn watched(name: &str) -> PathBuf {
    root().join("ui").join(name)
}

fn poll_ready(watcher: &mut UiAssetWorkspaceWatcher) -> UiAssetWorkspaceWatchPollReport {
    match watcher.begin_poll() {
        UiAssetWatchPollStart::Ready(report) => report,
        UiAssetWatchPollStart::Reconcile { .. } => {
            panic!("ordinary bounded path polling must not require reconcile")
        }
    }
}

fn layout_source(session_index: usize, import_count: usize) -> String {
    let imports = (0..import_count)
        .map(|import_index| format!(r#""res://ui/import-{session_index}-{import_index}.zui""#))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"
[asset]
kind = "layout"
id = "editor.test.reconcile.{session_index}"
version = 1
display_name = "Reconcile {session_index}"

[imports]
widgets = [{imports}]
styles = []

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
control_id = "Root"
"#
    )
}

fn reconcile_sessions(
    session_count: usize,
    imports_per_session: usize,
) -> BTreeMap<ViewInstanceId, UiAssetWorkspaceEntry> {
    (0..session_count)
        .map(|session_index| {
            let source = layout_source(session_index, imports_per_session);
            let route = UiAssetEditorRoute::new(
                format!("res://ui/session-{session_index}.zui"),
                UiAssetKind::Layout,
                UiAssetEditorMode::Design,
            );
            let session =
                UiAssetEditorSession::from_source(route, source.clone(), UiSize::new(320.0, 180.0))
                    .unwrap();
            (
                ViewInstanceId::new(format!("reconcile-{session_index}")),
                UiAssetWorkspaceEntry::new(
                    watched(&format!("session-{session_index}.zui")),
                    source,
                    session,
                ),
            )
        })
        .collect()
}

#[test]
fn budget_rejects_zero_capacity_count_and_time() {
    assert_eq!(
        UiAssetWatchBudget::try_new(0, 1, Duration::from_millis(1)),
        Err(UiAssetWatchBudgetError::ZeroPendingPathCapacity)
    );
    assert_eq!(
        UiAssetWatchBudget::try_new(1, 0, Duration::from_millis(1)),
        Err(UiAssetWatchBudgetError::ZeroPathsPerPoll)
    );
    assert_eq!(
        UiAssetWatchBudget::try_new(1, 1, Duration::ZERO),
        Err(UiAssetWatchBudgetError::ZeroPollTime)
    );
}

#[test]
fn ten_thousand_same_path_events_retain_one_latest_entry() {
    let ingress = UiAssetWatchIngressHandle::new(8);
    let observed_at = Instant::now();
    ingress.record_paths_at(
        std::iter::repeat_n(watched("same.zui"), 10_000),
        observed_at,
    );

    let snapshot = ingress.snapshot(observed_at);
    assert_eq!(snapshot.pending_path_count, 1);
    assert_eq!(snapshot.received_path_count, 10_000);
    assert_eq!(snapshot.coalesced_path_count, 9_999);
    assert_eq!(snapshot.overflow_count, 0);
}

#[test]
fn overflow_is_bounded_and_starts_a_reconcile_cursor() {
    let budget = UiAssetWatchBudget::try_new(2, 2, Duration::from_secs(1)).unwrap();
    let mut watcher = UiAssetWorkspaceWatcher::without_notify_for_test(vec![root()], budget);
    watcher.record_paths_for_test([watched("one.zui"), watched("two.zui"), watched("three.zui")]);

    let UiAssetWatchPollStart::Reconcile { cursor, allowance } = watcher.begin_poll() else {
        panic!("overflow must start a reconcile cursor");
    };
    let report = watcher.finish_reconcile(Some(cursor), allowance, BTreeSet::new());
    assert!(report.changed_asset_ids.is_empty());
    assert_eq!(report.diagnostics.pending_path_count, 0);
    assert_eq!(report.diagnostics.overflow_count, 1);
    assert!(report.diagnostics.reconcile_cursor_active);
}

#[test]
fn restoring_a_polled_suffix_after_callback_refill_preserves_capacity() {
    let ingress = UiAssetWatchIngressHandle::new(2);
    ingress.record_paths([watched("old-one.zui"), watched("old-two.zui")]);
    let drained = ingress.drain_paths(2);
    assert_eq!(drained.len(), 2);

    ingress.record_paths([watched("new-one.zui"), watched("new-two.zui")]);
    ingress.restore_paths_front(drained);

    let snapshot = ingress.snapshot(Instant::now());
    assert_eq!(snapshot.pending_path_count, 2);
    assert_eq!(snapshot.overflow_count, 1);
    assert!(ingress.take_overflow());
}

#[test]
fn poll_count_budget_retains_remaining_paths_for_the_next_tick() {
    let budget = UiAssetWatchBudget::try_new(8, 2, Duration::from_secs(1)).unwrap();
    let mut watcher = UiAssetWorkspaceWatcher::without_notify_for_test(vec![root()], budget);
    watcher.record_paths_for_test([watched("one.zui"), watched("two.zui"), watched("three.zui")]);

    let first = poll_ready(&mut watcher);
    assert_eq!(first.changed_asset_ids.len(), 2);
    assert_eq!(first.diagnostics.pending_path_count, 1);
    assert!(first.diagnostics.budget_exhausted);

    let second = poll_ready(&mut watcher);
    assert_eq!(second.changed_asset_ids, ["res://ui/three.zui"]);
    assert_eq!(second.diagnostics.pending_path_count, 0);
    assert!(!second.diagnostics.budget_exhausted);
}

#[test]
fn reconcile_enumeration_shares_the_item_budget_across_sessions_and_imports() {
    let sessions = reconcile_sessions(32, 8);
    let budget = UiAssetWatchBudget::try_new(1, 3, Duration::from_secs(1)).unwrap();
    let mut cursor = UiAssetWatchReconcileCursor::default();
    let mut resolved = BTreeSet::new();
    let mut batch_count = 0usize;

    loop {
        let mut allowance = budget.start_poll();
        let (batch, completed) =
            collect_ui_asset_reconcile_batch(&sessions, &mut cursor, &mut allowance);
        assert!(batch.len() <= 3);
        assert!(allowance.consumed_items_for_test() <= 3);
        resolved.extend(batch);
        batch_count += 1;
        if completed {
            break;
        }
    }

    assert_eq!(resolved.len(), 32 * 9);
    assert!(batch_count > 32);
}

#[test]
fn expired_reconcile_deadline_retains_the_cursor_without_cloning_imports() {
    let sessions = reconcile_sessions(4, 16);
    let budget = UiAssetWatchBudget::try_new(1, 64, Duration::from_secs(1)).unwrap();
    let mut cursor = UiAssetWatchReconcileCursor::default();
    let mut allowance = budget.start_poll();
    allowance.expire_for_test();

    let (batch, completed) =
        collect_ui_asset_reconcile_batch(&sessions, &mut cursor, &mut allowance);
    assert!(batch.is_empty());
    assert!(!completed);
    assert_eq!(allowance.consumed_items_for_test(), 0);
    assert_eq!(cursor.next_item_index, 0);
}

#[test]
fn second_overflow_is_visible_in_the_report_after_reconcile_work() {
    let budget = UiAssetWatchBudget::try_new(1, 4, Duration::from_secs(1)).unwrap();
    let mut watcher = UiAssetWorkspaceWatcher::without_notify_for_test(vec![root()], budget);
    watcher.record_paths_for_test([watched("first.zui"), watched("first-overflow.zui")]);
    let UiAssetWatchPollStart::Reconcile {
        cursor: _,
        allowance,
    } = watcher.begin_poll()
    else {
        panic!("first overflow must start reconcile");
    };

    watcher.record_paths_for_test([watched("second.zui"), watched("second-overflow.zui")]);
    let report = watcher.finish_reconcile(None, allowance, BTreeSet::new());
    assert_eq!(report.diagnostics.pending_path_count, 1);
    assert_eq!(report.diagnostics.overflow_count, 2);
    assert!(report.diagnostics.reconcile_cursor_active);
}

#[test]
fn diagnostics_report_oldest_pending_age_from_the_real_queue_head() {
    let ingress = UiAssetWatchIngressHandle::new(4);
    let observed_at = Instant::now();
    let first_seen_at = observed_at - Duration::from_millis(50);
    ingress.record_paths_at([watched("aged.zui")], first_seen_at);

    let snapshot = ingress.snapshot(observed_at);
    assert_eq!(snapshot.oldest_pending_age, Duration::from_millis(50));
}

#[test]
fn watcher_reports_a_res_uri_for_an_event_created_in_the_second_manifest_root() {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
    use zircon_runtime::asset::AssetUri;
    use zircon_runtime_interface::project::RelPath;

    let project_root = std::env::temp_dir().join(format!(
        "zircon-editor-dual-root-watcher-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&project_root).unwrap();
    let paths = ProjectPaths::from_root(&project_root).unwrap();
    let mut manifest = ProjectManifest::new(
        "Watcher",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    manifest.asset_roots = vec![
        RelPath::parse("game-assets").unwrap(),
        RelPath::parse("shared-assets").unwrap(),
    ];
    manifest.save(paths.manifest_path()).unwrap();
    let project = ProjectManager::open(&project_root).unwrap();
    let mut watcher = UiAssetWorkspaceWatcher::start(&project).unwrap();
    let changed = project_root.join("shared-assets/ui/second-root.zui");
    fs::create_dir_all(changed.parent().unwrap()).unwrap();
    fs::write(&changed, "version = 2").unwrap();

    let mut changed_asset_ids = Vec::new();
    for _ in 0..100 {
        match watcher.begin_poll() {
            UiAssetWatchPollStart::Ready(report) => {
                changed_asset_ids.extend(report.changed_asset_ids)
            }
            UiAssetWatchPollStart::Reconcile { .. } => {
                panic!("single file event must not overflow the watcher")
            }
        }
        if !changed_asset_ids.is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(changed_asset_ids.contains(&"res://ui/second-root.zui".to_string()));
    drop(watcher);
    let _ = fs::remove_dir_all(project_root);
}
