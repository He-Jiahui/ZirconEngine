#[cfg(windows)]
use std::path::Path;

use super::{
    invalid_runtime_project_root_error, missing_runtime_project_manifest_error,
    parse_runtime_session_startup_args, RuntimeSessionProfile,
};

#[cfg(windows)]
#[test]
fn runtime_project_argument_errors_hide_windows_verbatim_roots() {
    let root = Path::new(r"\\?\C:\projects\missing");

    assert!(invalid_runtime_project_root_error(root)
        .to_string()
        .contains("requested=C:\\projects\\missing"));
    assert!(missing_runtime_project_manifest_error(root)
        .to_string()
        .contains("requested=C:\\projects\\missing"));
}

#[test]
fn runtime_session_args_default_to_runtime_profile() {
    let parsed = parse_runtime_session_startup_args(["--log-level=debug".to_string()]).unwrap();

    assert_eq!(parsed.profile, RuntimeSessionProfile::Runtime);
    assert_eq!(parsed.profile.as_bytes(), b"runtime");
    assert_eq!(parsed.project_root, None);
    assert_eq!(parsed.play_scene, None);
    assert_eq!(parsed.play_report_pipe, None);
    assert!(!parsed.reference_cpu_presenter);
    assert!(!parsed.help_requested);
    assert_eq!(parsed.remaining_args, ["--log-level=debug"]);
}

#[test]
fn runtime_session_args_require_explicit_reference_cpu_presenter_opt_in() {
    let parsed =
        parse_runtime_session_startup_args(["--reference-cpu-presenter".to_string()]).unwrap();

    assert!(parsed.reference_cpu_presenter);
    assert!(parsed.remaining_args.is_empty());
}

#[test]
fn runtime_session_args_reject_duplicate_reference_cpu_presenter_opt_ins() {
    let error = parse_runtime_session_startup_args([
        "--reference-cpu-presenter".to_string(),
        "--reference-cpu-presenter".to_string(),
    ])
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "runtime startup diagnostic: component=runtime_app argument=--reference-cpu-presenter requested=<multiple> cause=reference CPU presenter was enabled more than once recovery=provide --reference-cpu-presenter at most once"
    );
}

#[test]
fn runtime_session_args_strip_space_separated_profile() {
    let parsed = parse_runtime_session_startup_args([
        "--runtime-session-profile".to_string(),
        "dev".to_string(),
        "--leftover".to_string(),
    ])
    .unwrap();

    assert_eq!(parsed.profile, RuntimeSessionProfile::Dev);
    assert_eq!(parsed.profile.as_bytes(), b"dev");
    assert_eq!(parsed.project_root, None);
    assert_eq!(parsed.remaining_args, ["--leftover"]);
}

#[test]
fn runtime_session_args_strip_equals_profile() {
    let parsed =
        parse_runtime_session_startup_args(["--runtime-session-profile=headless".to_string()])
            .unwrap();

    assert_eq!(parsed.profile, RuntimeSessionProfile::Headless);
    assert_eq!(parsed.profile.as_bytes(), b"headless");
    assert_eq!(parsed.project_root, None);
    assert!(!parsed.help_requested);
    assert!(parsed.remaining_args.is_empty());
}

#[test]
fn runtime_session_args_accept_pipelined_runtime_profile() {
    let parsed = parse_runtime_session_startup_args([
        "--runtime-session-profile=runtime-pipelined".to_string(),
    ])
    .unwrap();

    assert_eq!(parsed.profile, RuntimeSessionProfile::RuntimePipelined);
    assert_eq!(parsed.profile.as_bytes(), b"runtime-pipelined");
    assert!(parsed.remaining_args.is_empty());
}

#[test]
fn runtime_session_args_strip_space_separated_project_root() {
    let parsed = parse_runtime_session_startup_args([
        "--project".to_string(),
        "examples/vampire".to_string(),
        "--runtime-session-profile=dev".to_string(),
    ])
    .unwrap();

    assert_eq!(parsed.profile, RuntimeSessionProfile::Dev);
    assert_eq!(
        parsed.project_root.as_deref(),
        Some(std::path::Path::new("examples/vampire"))
    );
    assert!(parsed.remaining_args.is_empty());
}

#[test]
fn runtime_session_args_strip_equals_project_root() {
    let parsed =
        parse_runtime_session_startup_args(["--project=examples/vampire".to_string()]).unwrap();

    assert_eq!(
        parsed.project_root.as_deref(),
        Some(std::path::Path::new("examples/vampire"))
    );
    assert!(parsed.remaining_args.is_empty());
}

#[test]
fn runtime_session_args_accept_project_relative_play_startup_values() {
    let parsed = parse_runtime_session_startup_args([
        "--project".to_string(),
        ".".to_string(),
        "--play-scene=.zircon/play/42/play-scene.zrscene.json".to_string(),
        "--play-report-pipe".to_string(),
        "zircon-play-report-42".to_string(),
    ])
    .unwrap();

    assert_eq!(
        parsed.play_scene.as_ref().map(|path| path.as_str()),
        Some(".zircon/play/42/play-scene.zrscene.json")
    );
    assert_eq!(
        parsed.play_report_pipe.as_deref(),
        Some("zircon-play-report-42")
    );
    assert!(parsed.remaining_args.is_empty());
}

#[test]
fn runtime_session_args_strip_help_request() {
    let parsed = parse_runtime_session_startup_args([
        "--help".to_string(),
        "--runtime-session-profile=dev".to_string(),
        "-h".to_string(),
    ])
    .unwrap();

    assert_eq!(parsed.profile, RuntimeSessionProfile::Dev);
    assert!(parsed.help_requested);
    assert!(parsed.remaining_args.is_empty());
}

#[test]
fn runtime_session_help_lists_profiles_and_diagnostic_inputs() {
    for expected in [
        "--runtime-session-profile",
        "runtime",
        "runtime-pipelined",
        "editor",
        "dev",
        "minimal",
        "headless",
        "--project",
        "--reference-cpu-presenter",
        "--log-level",
        "--log-filter",
        "ZIRCON_RUNTIME_LIBRARY",
        "relative to the product executable",
        "relative paths resolve from the open project root, or launch directory without a project",
        "ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME",
        "ZIRCON_RUNTIME_EXIT_AFTER_PRESENTED_FRAMES",
        "ZIRCON_LOG_FILTER",
        "ZIRCON_LOG",
        "RUST_LOG",
        "ZIRCON_LOG_LEVEL",
    ] {
        assert!(
            super::RUNTIME_SESSION_STARTUP_HELP.contains(expected),
            "runtime help should mention `{expected}`"
        );
    }
}

#[test]
fn runtime_session_args_reject_duplicate_project_roots() {
    let error = parse_runtime_session_startup_args([
        "--project=examples/vampire".to_string(),
        "--project".to_string(),
        "examples/other".to_string(),
    ])
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "runtime startup diagnostic: component=runtime_app argument=--project requested=<multiple> cause=project root was provided more than once recovery=provide exactly one project root after --project"
    );
}

#[test]
fn runtime_session_args_reject_missing_project_root() {
    let error = parse_runtime_session_startup_args(["--project".to_string()]).unwrap_err();

    assert_eq!(
        error.to_string(),
        "runtime startup diagnostic: component=runtime_app argument=--project requested=<missing> cause=missing project root path recovery=provide an existing Zircon project root after --project"
    );
}

#[test]
fn runtime_session_args_reject_empty_project_root() {
    for args in [
        vec!["--project=".to_string()],
        vec!["--project".to_string(), "  ".to_string()],
        vec!["--project=  ".to_string()],
    ] {
        let error = parse_runtime_session_startup_args(args).unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_app argument=--project requested=<empty> cause=missing project root path recovery=provide an existing Zircon project root after --project"
        );
    }
}

#[test]
fn runtime_session_args_reject_duplicate_profiles() {
    let error = parse_runtime_session_startup_args([
        "--runtime-session-profile=dev".to_string(),
        "--runtime-session-profile".to_string(),
        "runtime".to_string(),
    ])
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "runtime startup diagnostic: component=runtime_app argument=--runtime-session-profile requested=<multiple> cause=runtime session profile was provided more than once recovery=provide exactly one --runtime-session-profile value"
    );
}

#[test]
fn runtime_session_args_reject_missing_profile_value() {
    let error =
        parse_runtime_session_startup_args(["--runtime-session-profile".to_string()]).unwrap_err();

    assert_eq!(
        error.to_string(),
        "runtime startup diagnostic: component=runtime_app argument=--runtime-session-profile requested=<missing> cause=missing runtime session profile recovery=provide runtime, runtime-pipelined, editor, dev, minimal, or headless after --runtime-session-profile"
    );
}

#[test]
fn runtime_session_args_reject_empty_profile_value() {
    for args in [
        vec!["--runtime-session-profile=".to_string()],
        vec!["--runtime-session-profile".to_string(), "  ".to_string()],
        vec!["--runtime-session-profile=  ".to_string()],
    ] {
        let error = parse_runtime_session_startup_args(args).unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_app argument=--runtime-session-profile requested=<empty> cause=missing runtime session profile recovery=provide runtime, runtime-pipelined, editor, dev, minimal, or headless after --runtime-session-profile"
        );
    }
}

#[test]
fn runtime_session_args_reject_unknown_profile_value() {
    let error =
        parse_runtime_session_startup_args(["--runtime-session-profile=debug-tools".to_string()])
            .unwrap_err();

    assert_eq!(
        error.to_string(),
        "runtime startup diagnostic: component=runtime_app argument=--runtime-session-profile requested=debug-tools cause=unsupported runtime session profile recovery=choose runtime, runtime-pipelined, editor, dev, minimal, or headless"
    );
}

#[test]
fn runtime_session_args_reject_invalid_play_scene_paths() {
    for value in [
        "",
        "  ",
        "C:\\project\\scene.zrscene.json",
        "/project/scene.zrscene.json",
        "../scene.zrscene.json",
    ] {
        let error =
            parse_runtime_session_startup_args(["--play-scene".to_string(), value.to_string()])
                .unwrap_err();

        assert!(
            error.to_string().contains("argument=--play-scene"),
            "expected Play scene diagnostic for {value:?}: {error}"
        );
    }
}

#[test]
fn runtime_session_args_reject_duplicate_and_missing_play_startup_values() {
    for args in [
        vec!["--play-scene".to_string()],
        vec![
            "--play-scene=a.zrscene.json".to_string(),
            "--play-scene".to_string(),
            "b.zrscene.json".to_string(),
        ],
        vec!["--play-report-pipe".to_string()],
        vec![
            "--play-report-pipe=one".to_string(),
            "--play-report-pipe=two".to_string(),
        ],
    ] {
        assert!(parse_runtime_session_startup_args(args).is_err());
    }
}
