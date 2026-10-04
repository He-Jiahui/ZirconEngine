use std::fs;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use super::SupervisedChild;

const PROCESS_TREE_FIXTURE_MARKER: &str = "ZIRCON_HUB_PROCESS_TREE_FIXTURE_MARKER";

#[test]
fn completed_root_still_owns_and_terminates_its_descendants() {
    let fixture_root = std::env::current_dir()
        .unwrap_or_default()
        .join(".codex")
        .join("scratch")
        .join(format!("hub-process-tree-fixture-{}", std::process::id()));
    let marker = fixture_root.join("descendant-survived.txt");
    fs::create_dir_all(&fixture_root).expect("create process-tree fixture root");
    let _ = fs::remove_file(&marker);

    let mut command = Command::new(std::env::current_exe().expect("current test executable"));
    command
        .args([
            "--exact",
            "process::child_supervisor::tests::process_tree_root_fixture",
            "--ignored",
        ])
        .env(PROCESS_TREE_FIXTURE_MARKER, &marker);
    let mut child = SupervisedChild::spawn(&mut command, "process-tree fixture")
        .expect("spawn supervised fixture root");

    let deadline = Instant::now() + Duration::from_secs(5);
    while child.try_wait().expect("poll fixture root").is_none() {
        assert!(Instant::now() < deadline, "fixture root did not exit");
        thread::sleep(Duration::from_millis(10));
    }
    child
        .terminate_tree_and_reap()
        .expect("terminate descendants after root exit");

    thread::sleep(Duration::from_millis(900));
    assert!(
        !marker.exists(),
        "a descendant escaped the persistent process-tree lease"
    );
    fs::remove_dir_all(fixture_root).expect("remove process-tree fixture root");
}

#[test]
fn dropping_completed_root_terminates_its_descendants() {
    let fixture_root = std::env::current_dir()
        .unwrap_or_default()
        .join(".codex")
        .join("scratch")
        .join(format!(
            "hub-process-tree-drop-fixture-{}",
            std::process::id()
        ));
    let marker = fixture_root.join("descendant-survived.txt");
    fs::create_dir_all(&fixture_root).expect("create process-tree drop fixture root");
    let _ = fs::remove_file(&marker);

    let mut command = Command::new(std::env::current_exe().expect("current test executable"));
    command
        .args([
            "--exact",
            "process::child_supervisor::tests::process_tree_root_fixture",
            "--ignored",
        ])
        .env(PROCESS_TREE_FIXTURE_MARKER, &marker);
    let mut child = SupervisedChild::spawn(&mut command, "process-tree drop fixture")
        .expect("spawn supervised fixture root");

    let deadline = Instant::now() + Duration::from_secs(5);
    while child.try_wait().expect("poll fixture root").is_none() {
        assert!(Instant::now() < deadline, "fixture root did not exit");
        thread::sleep(Duration::from_millis(10));
    }
    drop(child);

    thread::sleep(Duration::from_millis(900));
    assert!(
        !marker.exists(),
        "a dropped process-tree lease left a descendant alive"
    );
    fs::remove_dir_all(fixture_root).expect("remove process-tree drop fixture root");
}

#[test]
#[ignore = "child fixture for completed_root_still_owns_and_terminates_its_descendants"]
fn process_tree_root_fixture() {
    Command::new(std::env::current_exe().expect("current test executable"))
        .args([
            "--exact",
            "process::child_supervisor::tests::process_tree_descendant_fixture",
            "--ignored",
        ])
        .spawn()
        .expect("spawn long-lived descendant");
}

#[test]
#[ignore = "descendant fixture for completed_root_still_owns_and_terminates_its_descendants"]
fn process_tree_descendant_fixture() {
    thread::sleep(Duration::from_millis(700));
    let marker = std::env::var_os(PROCESS_TREE_FIXTURE_MARKER)
        .expect("process-tree fixture marker must be inherited");
    fs::write(marker, b"escaped").expect("write escaped descendant marker");
}
