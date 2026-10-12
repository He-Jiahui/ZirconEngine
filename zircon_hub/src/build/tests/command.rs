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
            .join("build")
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

#[cfg(windows)]
#[test]
fn first_build_uses_the_managed_default_and_current_tool_entry() {
    let settings = crate::settings::HubSettings::default();
    let source_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Hub belongs to the engine workspace")
        .to_path_buf();
    let command = BuildCommand::for_editor_runtime(&BuildCommandOptions::new(
        &settings.python_path,
        &settings.cargo_path,
        &source_dir,
        &settings.default_build_output_dir,
        settings.build_profile,
        Some(settings.jobs),
    ));

    assert!(PathBuf::from(&command.args[0]).is_file());
    let output_arg = command.args.iter().position(|arg| arg == "--out").unwrap() + 1;
    let output = PathBuf::from(&command.args[output_arg]);
    assert!(output.starts_with(PathBuf::from("E:/cargo-targets")));
    assert!(command
        .capture_dir
        .starts_with(PathBuf::from("E:/cargo-targets")));
}
