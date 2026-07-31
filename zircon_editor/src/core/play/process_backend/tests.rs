use std::{ffi::OsString, path::Path};

use super::command::PlayProcessCommand;

#[test]
fn process_command_uses_runtime_profile_scene_and_report_pipe_contract() {
    let command = PlayProcessCommand::new(
        "zircon_runtime",
        "project",
        "project/.zircon/play/1/play-scene.zrscene.json",
        "zircon-play-report-1",
    );
    let arguments = command.arguments();

    let expected = vec![
        "--project",
        "project",
        "--runtime-session-profile",
        "runtime",
        "--play-scene",
        "project/.zircon/play/1/play-scene.zrscene.json",
        "--play-report-pipe",
        "zircon-play-report-1",
    ]
    .into_iter()
    .map(OsString::from)
    .collect::<Vec<_>>();
    assert_eq!(arguments, expected);
    assert_eq!(command.executable(), Path::new("zircon_runtime"));
}
