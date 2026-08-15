#[test]
fn editor_help_flags_are_recognized_before_any_startup_mode() {
    use crate::entry::cli::{EditorLaunchArgs, EditorLaunchRoute};

    for args in [["--help"], ["-h"]] {
        assert!(matches!(
            EditorLaunchArgs::parse(args).unwrap().route().unwrap(),
            EditorLaunchRoute::Help
        ));
    }
    assert!(matches!(
        EditorLaunchArgs::parse(["--project", "E:/Projects/ZirconMvp"])
            .unwrap()
            .route()
            .unwrap(),
        EditorLaunchRoute::Gui(_)
    ));
}

#[test]
fn editor_help_returns_success_without_creating_an_editor_host() {
    let exit_code = super::super::EntryRunner::run_editor_with_args_exit_code(["--help"]).expect(
        "editor help should return without requiring a project, runtime library, or window",
    );

    assert_eq!(exit_code, 0);
}

#[test]
fn editor_help_documents_product_startup_and_first_frame_automation() {
    for expected in [
        "--project <path>",
        "--scene <res://path.scene.toml>",
        "--layout <preset-id>",
        "--create-project",
        "--run <commandlet>",
        "ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG",
        "ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME",
        "ZIRCON_RUNTIME_LIBRARY",
    ] {
        assert!(
            super::super::EDITOR_STARTUP_HELP.contains(expected),
            "editor help should mention `{expected}`"
        );
    }
}

#[test]
fn editor_help_exposes_run_as_the_only_headless_entry() {
    for retired in [
        "--operation <id>",
        "--list-operations",
        "--operation-history",
        "--headless",
    ] {
        assert!(
            !super::super::EDITOR_STARTUP_HELP.contains(retired),
            "editor help must not retain the retired `{retired}` CLI entry"
        );
    }
}

#[test]
fn editor_help_returns_before_commandlet_or_gui_startup() {
    let source = include_str!("../../editor.rs");
    let launch_args = source
        .find("let launch_args = EditorLaunchArgs::parse(args)?;")
        .expect("editor startup should parse the unified launch intent first");
    let log_initialization = source
        .find("initialize_process_log_with_config(")
        .expect("editor startup should initialize diagnostics before routing");
    let route = source
        .find("let (gui_startup_request, startup_scene_uri, startup_layout_preset) =")
        .expect("editor startup should consume the typed launch route");
    let help = source
        .find("EditorLaunchRoute::Help => {")
        .expect("editor startup should handle help inside the typed route");
    let commandlet = source
        .find("EditorLaunchRoute::Commandlet(request) => {")
        .expect("editor startup should route commandlets after help handling");

    assert!(
        launch_args < log_initialization && log_initialization < route && help < commandlet,
        "editor --help must not instantiate commandlet, GUI, or runtime startup paths"
    );
}
