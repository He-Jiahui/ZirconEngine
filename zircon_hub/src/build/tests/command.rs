use super::*;

#[test]
fn build_command_matches_staged_tool_contract() {
    let command = BuildCommand::for_editor_runtime(&BuildCommandOptions::new(
        "python",
        "cargo-nextest",
        "E:/Git/ZirconEngine",
        "E:/build out",
        BuildProfile::Debug,
        Some(4),
    ));

    assert_eq!(command.program, PathBuf::from("python"));
    assert_eq!(command.cwd, PathBuf::from("E:/Git/ZirconEngine"));
    assert_eq!(
        command.capture_dir,
        PathBuf::from("E:/.zircon-hub/build-capture")
    );
    assert_eq!(
        PathBuf::from(&command.args[0]),
        PathBuf::from("E:/Git/ZirconEngine")
            .join("tools")
            .join("zircon_build.py")
    );
    assert_eq!(
        &command.args[1..],
        [
            "--targets",
            "editor,runtime",
            "--out",
            "E:/build out",
            "--mode",
            "debug",
            "--cargo",
            "cargo-nextest",
            "--jobs",
            "4",
        ]
    );
}
