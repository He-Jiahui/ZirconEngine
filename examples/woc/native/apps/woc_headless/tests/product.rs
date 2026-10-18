#![cfg(feature = "engine-host")]

use std::fs;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn headless_product_executes_authored_world_and_replays_retained_checkpoint() {
    let logs = tempfile::tempdir().unwrap();
    let stdout_path = logs.path().join("stdout.json");
    let stderr_path = logs.path().join("stderr.log");
    let mut child = Command::new(env!("CARGO_BIN_EXE_woc_headless"))
        .args(["--ticks", "2", "--verify-replay"])
        .stdout(Stdio::from(fs::File::create(&stdout_path).unwrap()))
        .stderr(Stdio::from(fs::File::create(&stderr_path).unwrap()))
        .spawn()
        .unwrap();
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if started.elapsed() > Duration::from_secs(120) {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("headless product exceeded its 120-second startup and execution deadline");
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    let stderr = fs::read_to_string(stderr_path).unwrap();
    assert!(
        status.success(),
        "headless product failed: {status}\n{stderr}"
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(stdout_path).unwrap()).unwrap();
    assert_eq!(report["identity"]["backend"], "zr_vm:project");
    assert_eq!(report["identity"]["role"], "headless");
    assert_eq!(report["committedTick"], 3);
    assert_eq!(report["replayVerified"], true);
    assert_eq!(report["presentationBytes"], 0);
    assert_eq!(report["presentationReady"], false);
    assert_eq!(report["renderedFrames"], 0);
}
