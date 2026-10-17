use std::time::Duration;

use zircon_runtime_interface::hub_protocol::{
    HubRecentProjectV1, HubRecentProjectsStore, HubRecentProjectsWritePolicy,
};

use super::{
    load_shared_recent_projects, reconcile_shared_recent_projects,
    reconcile_shared_recent_projects_snapshot, SharedRecentProjectsSnapshot,
};
use crate::projects::RecentProject;

#[test]
fn reconciliation_merges_hub_history_with_the_shared_v1_registry() {
    let root = temporary_root("merge");
    let registry_path = root.join("recent_projects.json");

    let first = reconcile_shared_recent_projects(
        &registry_path,
        &[],
        &[RecentProject::fixture("Old", "E:/Projects/Game", 1)],
    )
    .expect("write initial shared history");
    let second = reconcile_shared_recent_projects(
        &registry_path,
        &first,
        &[
            RecentProject::fixture("Current", "e:/projects/game/", 9),
            RecentProject::fixture("Other", "E:/Projects/Other", 2),
        ],
    )
    .expect("merge shared history");

    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 2);
    assert_eq!(second[0].summary.name, "Current");
    assert_eq!(
        load_shared_recent_projects(&registry_path).expect("read shared history")[0]
            .summary
            .name,
        "Current"
    );
    std::fs::remove_dir_all(root).expect("remove shared history fixture");
}

#[test]
fn stale_hub_snapshot_does_not_restore_an_editor_removed_project() {
    let root = temporary_root("delete");
    let registry_path = root.join("recent_projects.json");
    let initial = reconcile_shared_recent_projects(
        &registry_path,
        &[],
        &[RecentProject::fixture("Game", "E:/Projects/Game", 1)],
    )
    .expect("write initial shared history");
    let removed = reconcile_shared_recent_projects(&registry_path, &initial, &[])
        .expect("record Editor-side removal");
    let after_stale_hub_persist =
        reconcile_shared_recent_projects(&registry_path, &initial, &initial)
            .expect("reconcile stale Hub snapshot");

    assert!(removed.is_empty());
    assert!(after_stale_hub_persist.is_empty());
    std::fs::remove_dir_all(root).expect("remove shared history fixture");
}

#[test]
fn stale_hub_delete_preserves_an_externally_updated_project() {
    let root = temporary_root("stale-delete");
    let registry_path = root.join("recent_projects.json");
    let initial = reconcile_shared_recent_projects_snapshot(
        &registry_path,
        &SharedRecentProjectsSnapshot::default(),
        &[RecentProject::fixture("Game", "E:/Projects/Game", 1)],
    )
    .expect("write initial shared history");
    HubRecentProjectsStore::new(&registry_path)
        .update(
            HubRecentProjectsWritePolicy::with_timeout(Duration::from_millis(50)),
            |registry| {
                registry.record(HubRecentProjectV1::new(
                    RecentProject::fixture("External", "E:/Projects/Game", 9).summary,
                    "E:/Projects/Game",
                    9,
                )?)
            },
        )
        .expect("external editor update");

    let reconciled = reconcile_shared_recent_projects_snapshot(&registry_path, &initial, &[])
        .expect("stale Hub delete must rebase safely");

    assert_eq!(reconciled.projects()[0].summary.name, "External");
    std::fs::remove_dir_all(root).expect("remove shared history fixture");
}

fn temporary_root(label: &str) -> std::path::PathBuf {
    let target_directory = std::env::var_os("CARGO_TARGET_DIR").expect(
        "shared recent-project filesystem tests require coordinator-managed CARGO_TARGET_DIR",
    );
    std::path::PathBuf::from(target_directory).join(format!(
        "zircon-hub-shared-recents-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ))
}
