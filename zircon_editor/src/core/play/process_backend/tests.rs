use std::{ffi::OsString, path::Path};

use super::{command::PlayProcessCommand, ProcessPlayBackend, ProcessPlayBackendInstallError};
use zircon_runtime_interface::project::RelPath;

#[test]
fn process_backend_install_resolution_keeps_failures_typed() {
    let _: fn() -> Result<ProcessPlayBackend, ProcessPlayBackendInstallError> =
        ProcessPlayBackend::for_current_install;
}

#[test]
fn process_command_uses_runtime_profile_scene_and_report_pipe_contract() {
    let command = PlayProcessCommand::new(
        "zircon_runtime",
        "project",
        RelPath::parse(".zircon/play/1/play-scene.zrscene.json").unwrap(),
        "zircon-play-report-1",
    );
    let arguments = command.arguments();

    let expected = vec![
        "--project",
        ".",
        "--runtime-session-profile",
        "runtime",
        "--play-scene",
        ".zircon/play/1/play-scene.zrscene.json",
        "--play-report-pipe",
        "zircon-play-report-1",
    ]
    .into_iter()
    .map(OsString::from)
    .collect::<Vec<_>>();
    assert_eq!(arguments, expected);
    assert_eq!(command.executable(), Path::new("zircon_runtime"));
}

#[test]
fn process_command_configures_the_shared_process_tree_cancellation_domain() {
    let source = include_str!("command.rs");

    assert!(source.contains("configure_process_tree_cancellation(&mut command)"));
    assert!(source.contains("configure_process_tree_suspended_spawn(&mut command)"));
}

#[test]
fn process_backend_retains_child_ownership_when_stop_is_not_terminal() {
    let source = include_str!("mod.rs");
    let stop = source
        .split("fn stop(&self)")
        .nth(1)
        .and_then(|body| body.split("fn poll(&self)").next())
        .expect("process backend stop implementation");

    assert!(source.contains("enum ActivePlayProcess"));
    assert!(source.contains("Running(PlayChild)"));
    assert!(source.contains("Stopping"));
    assert!(stop.contains("ActivePlayProcess::Stopping"));
    assert!(stop.contains("failure.into_parts()"));
    assert!(stop.contains("ActivePlayProcess::Running(child)"));
}

#[test]
fn process_backend_does_not_report_running_without_an_owned_child() {
    let source = include_str!("mod.rs");
    let poll = source
        .split("fn poll(&self)")
        .nth(1)
        .and_then(|body| body.split("impl Drop").next())
        .expect("process backend poll implementation");

    assert!(poll.contains("runtime preview process is not active"));
    assert!(!poll
        .contains("return Ok(PlayBackendPoll::Running {\n                diagnostics: Vec::new()"));
}
