use std::{fs, path::PathBuf, time::SystemTime};

use super::ProjectCloudSyncLease;

fn project_root(label: &str) -> PathBuf {
    let target = PathBuf::from(std::env::var_os("CARGO_TARGET_DIR").expect("managed target"));
    let root = target.join(format!(
        "cloud-sync-lease-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(root.join(".zircon")).unwrap();
    root
}

#[test]
fn project_sync_lease_is_exclusive_and_released_on_drop() {
    let root = project_root("exclusive");
    let first = ProjectCloudSyncLease::acquire(&root).unwrap();
    assert!(ProjectCloudSyncLease::acquire(&root).is_err());

    drop(first);

    let next = ProjectCloudSyncLease::acquire(&root).unwrap();
    drop(next);
    fs::remove_dir_all(root).unwrap();
}
