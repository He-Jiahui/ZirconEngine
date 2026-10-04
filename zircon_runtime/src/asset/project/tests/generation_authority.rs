use std::fs::{self, File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::{lock_project_generation, GENERATION_LOCK_FILE_NAME};

const CHILD_PROJECT_ROOT: &str = "ZIRCON_GENERATION_AUTHORITY_CHILD_PROJECT_ROOT";
const CHILD_BLOCKED_MARKER: &str = "ZIRCON_GENERATION_AUTHORITY_CHILD_BLOCKED_MARKER";
const CHILD_ACQUIRED_MARKER: &str = "ZIRCON_GENERATION_AUTHORITY_CHILD_ACQUIRED_MARKER";
#[test]
fn generation_authority_lock_child_process() {
    let Some(root) = std::env::var_os(CHILD_PROJECT_ROOT).map(PathBuf::from) else {
        return;
    };
    let lock_path = root.join(".zircon").join(GENERATION_LOCK_FILE_NAME);
    let lock_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&lock_path)
        .expect("parent must create the persistent project lock file");
    match File::try_lock(&lock_file) {
        Err(TryLockError::WouldBlock) => {}
        Err(TryLockError::Error(error)) => {
            panic!("cross-process lock probe failed unexpectedly: {error}")
        }
        Ok(()) => {
            File::unlock(&lock_file).expect("unexpected lock probe must be released");
            panic!("a second process acquired the held project generation lock");
        }
    }
    fs::write(
        std::env::var_os(CHILD_BLOCKED_MARKER)
            .map(PathBuf::from)
            .expect("parent must provide the blocked marker path"),
        b"blocked",
    )
    .expect("child must report observing the held OS lock");

    let _generation = lock_project_generation(&root)
        .expect("child must acquire the project generation lock after release");
    fs::write(
        std::env::var_os(CHILD_ACQUIRED_MARKER)
            .map(PathBuf::from)
            .expect("parent must provide the acquired marker path"),
        b"acquired",
    )
    .expect("child must report acquiring the generation lock");
}

#[test]
fn project_generation_lock_serializes_separate_processes() {
    let root = unique_project_root();
    fs::create_dir_all(&root).expect("test project root must be created");
    let generation =
        lock_project_generation(&root).expect("parent must acquire the project generation lock");
    let blocked_marker = root.join(".zircon").join("child-blocked");
    let acquired_marker = root.join(".zircon").join("child-acquired");
    let mut child = Command::new(
        std::env::current_exe().expect("test harness executable path must be available"),
    )
    .arg("generation_authority_lock_child_process")
    .arg("--nocapture")
    .env(CHILD_PROJECT_ROOT, &root)
    .env(CHILD_BLOCKED_MARKER, &blocked_marker)
    .env(CHILD_ACQUIRED_MARKER, &acquired_marker)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .expect("child test process must start");

    wait_for_marker(&mut child, &blocked_marker);
    assert!(
        !acquired_marker.exists(),
        "child must remain blocked until the parent releases its lock"
    );
    drop(generation);
    wait_for_marker(&mut child, &acquired_marker);
    assert!(
        child
            .wait()
            .expect("child test process must exit")
            .success(),
        "child test process must complete successfully"
    );
    fs::remove_dir_all(root).expect("test project root must be cleaned up");
}

fn wait_for_marker(child: &mut Child, marker: &Path) {
    const WAIT_LIMIT: Duration = Duration::from_secs(10);
    let started = Instant::now();
    loop {
        if marker.is_file() {
            return;
        }
        if let Some(status) = child
            .try_wait()
            .expect("child test process status must be readable")
        {
            panic!(
                "child exited with {status} before creating {}",
                marker.display()
            );
        }
        assert!(
            started.elapsed() < WAIT_LIMIT,
            "child did not create {} before timeout",
            marker.display()
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn unique_project_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon-generation-authority-{}-{nonce}",
        std::process::id()
    ))
}
