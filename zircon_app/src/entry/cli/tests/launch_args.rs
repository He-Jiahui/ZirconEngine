use std::path::Path;

use super::{EditorGuiStartupRequestArgs, EditorLaunchArgs, EditorLaunchRoute};
use zircon_editor::{
    core::commandlet::{CommandletExitCode, CommandletStatus},
    EditorGuiStartupRequest,
};
use zircon_runtime::diagnostic_log::{DiagnosticLogFilter, DiagnosticLogLevel};
use zircon_runtime_interface::project::{
    ProjectActivationOperationId, ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
    ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource,
};

fn next_test_project_operation_id() -> ProjectActivationOperationId {
    ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
        .allocate()
        .expect("fresh test operation identity")
}

#[test]
fn unified_launch_args_route_run_to_the_editor_core_commandlet() {
    let route = EditorLaunchArgs::parse([
        "--run",
        "migrate-assets",
        "--project",
        "fixture",
        "--dry-run",
    ])
    .unwrap()
    .route()
    .unwrap();

    let EditorLaunchRoute::Commandlet(request) = route else {
        panic!("--run should route to the editor core commandlet");
    };
    assert_eq!(request.command(), "migrate-assets");
}

#[test]
fn unified_launch_args_preserve_authoring_automation_typed_arguments() {
    let route = EditorLaunchArgs::parse([
        "--run",
        "authoring-automation",
        "--project",
        "fixture-project",
        "--automation",
        "fixture-automation.json",
    ])
    .unwrap()
    .route()
    .unwrap();

    let EditorLaunchRoute::Commandlet(request) = route else {
        panic!("authoring automation should route through the editor core commandlet");
    };
    assert_eq!(request.command(), "authoring-automation");
    assert_eq!(request.project_root(), Some(Path::new("fixture-project")));
    assert_eq!(
        request.automation_path(),
        Some(Path::new("fixture-automation.json"))
    );
}

#[test]
fn unified_launch_args_preserve_json_parameter_errors_for_commandlets() {
    let route = EditorLaunchArgs::parse(["--run", "unknown", "--project", "fixture", "--dry-run"])
        .unwrap()
        .route()
        .unwrap();

    let EditorLaunchRoute::CommandletRejected(report) = route else {
        panic!("unknown --run target should return the commandlet JSON report");
    };
    assert_eq!(report.exit_code(), CommandletExitCode::InvalidArguments);
    assert_eq!(report.status(), CommandletStatus::InvalidArguments);
}

#[test]
fn unified_launch_args_initialize_diagnostics_before_routing_gui() {
    let launch_args =
        EditorLaunchArgs::parse(["--project", "fixture-project", "--log-level", "warn"]).unwrap();

    assert_eq!(
        launch_args.diagnostic_filter().minimum,
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Warn)
    );
    let route = launch_args.route().unwrap();
    assert!(matches!(route, EditorLaunchRoute::Gui(_)));
}

#[test]
fn unified_launch_args_carry_a_layout_preset_to_the_gui_host() {
    let route = EditorLaunchArgs::parse(["--project", "fixture-project", "--layout", "debug"])
        .unwrap()
        .route()
        .unwrap();

    let EditorLaunchRoute::Gui(intent) = route else {
        panic!("a GUI launch should carry the requested layout preset");
    };
    assert_eq!(intent.layout_preset(), Some("debug"));
    assert!(matches!(
        intent.into_parts().0,
        Some(EditorGuiStartupRequest::Project { .. })
    ));
}

#[test]
fn unified_launch_args_carry_a_project_scene_to_the_gui_host() {
    let route = EditorLaunchArgs::parse([
        "--project",
        "fixture-project",
        "--scene",
        "res://scenes/main.scene.toml",
    ])
    .unwrap()
    .route()
    .unwrap();

    let EditorLaunchRoute::Gui(intent) = route else {
        panic!("a GUI launch should carry the requested project scene");
    };
    assert_eq!(
        intent
            .startup_scene_uri()
            .map(ToString::to_string)
            .as_deref(),
        Some("res://scenes/main.scene.toml")
    );
    assert!(matches!(
        intent.into_parts().0,
        Some(EditorGuiStartupRequest::Project { .. })
    ));
}

#[test]
fn unified_launch_args_carry_a_verified_hub_handshake_to_the_gui_host() {
    let transmitted = ProjectLaunchIntent::open_existing(
        next_test_project_operation_id(),
        ProjectLaunchSource::Hub,
        ProjectLaunchProfile::Normal,
        "fixture-project",
    )
    .unwrap();
    let route = EditorLaunchArgs::parse([
        "--hub-protocol",
        "1",
        "--project-launch-intent",
        &serde_json::to_string(&transmitted).unwrap(),
        "--hub-session",
        "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52",
    ])
    .unwrap()
    .route()
    .unwrap();

    let EditorLaunchRoute::Gui(intent) = route else {
        panic!("a Hub launch should use the GUI route");
    };
    assert_eq!(
        intent
            .hub_handshake()
            .map(|handshake| handshake.session().to_string()),
        Some("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52".to_string())
    );
    assert!(matches!(
        intent.into_parts().0,
        Some(EditorGuiStartupRequest::Project { intent, .. }) if intent == transmitted
    ));
}

#[test]
fn gui_launch_intent_rejects_incomplete_invalid_or_unscoped_hub_handshakes() {
    for (args, expected) in [
        (
            vec![
                "--project".to_string(),
                "fixture".to_string(),
                "--hub-session".to_string(),
                "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52".to_string(),
            ],
            "--hub-session requires --hub-protocol 1",
        ),
        (
            vec!["--hub-protocol".to_string(), "1".to_string()],
            "--hub-protocol requires --hub-session",
        ),
        (
            vec![
                "--project".to_string(),
                "fixture".to_string(),
                "--hub-session".to_string(),
                "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52".to_string(),
                "--hub-protocol".to_string(),
                "2".to_string(),
            ],
            "unsupported Hub protocol version 2; expected 1",
        ),
        (
            vec![
                "--hub-session".to_string(),
                "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52".to_string(),
                "--hub-protocol".to_string(),
                "1".to_string(),
            ],
            "--hub-session requires --project-launch-intent",
        ),
    ] {
        let error = EditorGuiStartupRequestArgs::parse_intent(args).unwrap_err();
        assert_eq!(error.to_string(), expected);
    }

    let error = EditorGuiStartupRequestArgs::parse_intent([
        "--project".to_string(),
        "fixture".to_string(),
        "--hub-session".to_string(),
        "not-a-uuid".to_string(),
        "--hub-protocol".to_string(),
        "1".to_string(),
    ])
    .unwrap_err();
    assert!(error
        .to_string()
        .starts_with("--hub-session requires a canonical UUID v4 token:"));
}

#[test]
fn hub_handshake_preserves_the_transmitted_project_operation_id() {
    let transmitted = ProjectLaunchIntent::open_existing(
        next_test_project_operation_id(),
        ProjectLaunchSource::Hub,
        ProjectLaunchProfile::Safe,
        "E:/Projects/My Game",
    )
    .unwrap();
    let payload = serde_json::to_string(&transmitted).unwrap();

    let parsed = EditorGuiStartupRequestArgs::parse_intent([
        "--project-launch-intent".to_string(),
        payload,
        "--hub-session".to_string(),
        "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52".to_string(),
        "--hub-protocol".to_string(),
        "1".to_string(),
    ])
    .unwrap();

    let Some(EditorGuiStartupRequest::Project { intent, .. }) = parsed.into_parts().0 else {
        panic!("a Hub launch should preserve its project launch intent");
    };
    assert_eq!(intent.operation_id(), transmitted.operation_id());
    assert_eq!(intent.profile(), ProjectLaunchProfile::Safe);
}

#[test]
fn hub_handshake_rejects_legacy_project_arguments() {
    let error = EditorGuiStartupRequestArgs::parse_intent([
        "--project".to_string(),
        "fixture-project".to_string(),
        "--hub-session".to_string(),
        "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52".to_string(),
        "--hub-protocol".to_string(),
        "1".to_string(),
    ])
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "--hub-session requires --project-launch-intent"
    );
}

#[test]
fn hub_intent_diagnostics_redact_the_serialized_project_payload() {
    let transmitted = ProjectLaunchIntent::open_existing(
        next_test_project_operation_id(),
        ProjectLaunchSource::Cli,
        ProjectLaunchProfile::Normal,
        "E:/Private Projects/Secret Game",
    )
    .unwrap();
    let payload = serde_json::to_string(&transmitted).unwrap();

    let error = EditorLaunchArgs::parse([
        "--project-launch-intent".to_string(),
        payload,
        "--hub-session".to_string(),
        "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52".to_string(),
        "--hub-protocol".to_string(),
        "1".to_string(),
    ])
    .unwrap()
    .route()
    .unwrap_err();

    let diagnostic = error.to_string();
    assert!(diagnostic.contains("--project-launch-intent <project-launch-intent>"));
    assert!(!diagnostic.contains("E:/Private Projects/Secret Game"));
}

#[test]
fn gui_launch_intent_rejects_invalid_or_unscoped_project_scenes() {
    for (args, expected) in [
        (
            vec!["--scene".to_string()],
            "--scene requires a scene asset URI",
        ),
        (
            vec![
                "--project".to_string(),
                "fixture".to_string(),
                "--scene".to_string(),
                " ".to_string(),
            ],
            "--scene requires a non-empty scene asset URI",
        ),
        (
            vec![
                "--scene".to_string(),
                "res://scenes/main.scene.toml".to_string(),
            ],
            "--scene requires --project",
        ),
        (
            vec![
                "--project".to_string(),
                "fixture".to_string(),
                "--scene".to_string(),
                "res://scenes/one.scene.toml".to_string(),
                "--scene".to_string(),
                "res://scenes/two.scene.toml".to_string(),
            ],
            "--scene was provided more than once",
        ),
    ] {
        let error = EditorGuiStartupRequestArgs::parse_intent(args).unwrap_err();
        assert_eq!(error.to_string(), expected);
    }

    let error = EditorGuiStartupRequestArgs::parse_intent([
        "--project".to_string(),
        "fixture".to_string(),
        "--scene".to_string(),
        "not-an-asset-uri".to_string(),
    ])
    .unwrap_err();
    assert!(error
        .to_string()
        .starts_with("--scene requires a valid scene asset URI:"));
}

#[test]
fn gui_launch_intent_rejects_missing_empty_or_duplicate_layout_presets() {
    for (args, expected) in [
        (
            vec!["--layout".to_string()],
            "--layout requires a preset id",
        ),
        (
            vec!["--layout".to_string(), " ".to_string()],
            "--layout requires a non-empty preset id",
        ),
        (
            vec![
                "--layout".to_string(),
                "debug".to_string(),
                "--layout".to_string(),
                "authoring".to_string(),
            ],
            "--layout was provided more than once",
        ),
    ] {
        let error = EditorGuiStartupRequestArgs::parse_intent(args).unwrap_err();
        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn unified_launch_args_route_help_before_host_or_commandlet_construction() {
    let route = EditorLaunchArgs::parse(["--run", "plugin-list", "--help"])
        .unwrap()
        .route()
        .unwrap();

    assert!(matches!(route, EditorLaunchRoute::Help));
}
