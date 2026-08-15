use std::{ffi::OsString, path::Path};

use super::command::PlayProcessCommand;
use zircon_runtime_interface::project::RelPath;

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
