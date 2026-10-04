use std::fs;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};

use super::super::SessionGuardError;
use super::{session_mutex_name, SessionOwnershipLease};

static NEXT_TEMP_ROOT: AtomicU64 = AtomicU64::new(1);
const WINDOWS_ERROR_PRIVILEGE_NOT_HELD: i32 = 1314;

#[test]
fn session_mutex_name_resolves_directory_aliases_through_the_project_resolver() {
    let root = temporary_root("lease-alias");
    let physical = root.join("physical-project");
    let alias = root.join("project-alias");
    fs::create_dir_all(&physical).expect("create physical project root");
    create_directory_alias(&physical, &alias);

    assert_eq!(
        session_mutex_name(&physical).expect("resolve physical project root"),
        session_mutex_name(&alias).expect("resolve project directory alias")
    );

    fs::remove_dir_all(&root).expect("remove project alias fixture");
}

#[test]
fn session_mutex_name_resolves_directory_symbolic_link_aliases_through_the_project_resolver() {
    let root = temporary_root("lease-symbolic-link");
    let physical = root.join("physical-project");
    let alias = root.join("project-alias");
    fs::create_dir_all(&physical).expect("create physical project root");
    if !create_directory_symbolic_link(&physical, &alias) {
        fs::remove_dir_all(&root).expect("remove project symbolic-link fixture");
        return;
    }

    assert_eq!(
        session_mutex_name(&physical).expect("resolve physical project root"),
        session_mutex_name(&alias).expect("resolve project directory symbolic link")
    );
    let lease =
        SessionOwnershipLease::acquire(&physical, &physical.join(".zircon").join("session.lock"))
            .expect("acquire physical project lease");
    assert!(matches!(
        SessionOwnershipLease::acquire(&alias, &alias.join(".zircon").join("session.lock")),
        Err(SessionGuardError::AlreadyHeld { .. })
    ));
    drop(lease);

    fs::remove_dir_all(&root).expect("remove project symbolic-link fixture");
}

#[test]
fn session_lease_rejects_a_directory_alias_while_the_physical_project_is_held() {
    let root = temporary_root("lease-physical-alias");
    let physical = root.join("physical-project");
    let alias = root.join("project-alias");
    fs::create_dir_all(&physical).expect("create physical project root");
    create_directory_alias(&physical, &alias);

    let lease =
        SessionOwnershipLease::acquire(&physical, &physical.join(".zircon").join("session.lock"))
            .expect("acquire physical project lease");
    assert!(matches!(
        SessionOwnershipLease::acquire(&alias, &alias.join(".zircon").join("session.lock")),
        Err(SessionGuardError::AlreadyHeld { .. })
    ));
    drop(lease);

    fs::remove_dir_all(&root).expect("remove project alias fixture");
}

fn temporary_root(label: &str) -> std::path::PathBuf {
    std::env::current_dir()
        .expect("current directory should be available")
        .join("target")
        .join(format!(
            "zircon-editor-session-{label}-{}-{}",
            std::process::id(),
            NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed)
        ))
}

fn create_directory_alias(target: &std::path::Path, link: &std::path::Path) {
    let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
    let output = std::process::Command::new("cmd")
        .args(["/D", "/S", "/C"])
        .arg(command)
        .output()
        .expect("start mklink for project session alias fixture");
    assert!(
        output.status.success(),
        "create project session junction fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn create_directory_symbolic_link(target: &std::path::Path, link: &std::path::Path) -> bool {
    match std::os::windows::fs::symlink_dir(target, link) {
        Ok(()) => true,
        Err(error)
            if error.kind() == io::ErrorKind::PermissionDenied
                || error.raw_os_error() == Some(WINDOWS_ERROR_PRIVILEGE_NOT_HELD) =>
        {
            false
        }
        Err(error) => panic!("create project session symbolic-link fixture failed: {error}"),
    }
}
