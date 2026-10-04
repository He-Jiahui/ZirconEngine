use std::str::FromStr;

use super::*;
use crate::projects::{CreateProjectRequest, ProjectTemplateId};

#[test]
fn editor_launch_args_preserve_project_paths_with_spaces() {
    let request = EditorLaunchRequest::open_project("E:/Projects/My Game").unwrap();
    let operation_id = request.intent().operation_id();
    let command =
        EditorLaunchCommand::new("E:/Engine/ZirconEngine/zircon_editor.exe", request).unwrap();

    assert_eq!(command.args[0], PROJECT_LAUNCH_INTENT_ARGUMENT);
    let transmitted = serde_json::from_str::<ProjectLaunchIntent>(&command.args[1]).unwrap();
    assert_eq!(transmitted.operation_id(), operation_id);
    assert_eq!(transmitted.source(), ProjectLaunchSource::Hub);
    assert!(matches!(
        transmitted.target(),
        zircon_runtime_interface::project::ProjectLaunchTarget::OpenExisting { requested_path }
            if requested_path == Path::new("E:/Projects/My Game")
    ));
}

#[test]
fn editor_create_args_match_editor_startup_contract() {
    let command = EditorLaunchCommand::new(
        "zircon_editor.exe",
        EditorLaunchRequest::create_project(CreateProjectRequest::new(
            "My Game",
            "E:/Projects",
            ProjectTemplateId::RenderableEmpty,
        ))
        .unwrap(),
    )
    .unwrap();

    let transmitted = serde_json::from_str::<ProjectLaunchIntent>(&command.args[1]).unwrap();
    assert!(matches!(
        transmitted.target(),
        zircon_runtime_interface::project::ProjectLaunchTarget::CreateProject {
            project_name,
            location,
            template: zircon_runtime_interface::project::ProjectTemplateId::RenderableEmpty,
        } if project_name == "My Game" && location == Path::new("E:/Projects")
    ));
}

#[test]
fn hub_handshake_arguments_use_the_typed_token_and_protocol_v1() {
    let token = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
        .expect("parse deterministic test token");
    let request = EditorLaunchRequest::open_project("E:/Projects/My Game").unwrap();
    let operation_id = request.intent().operation_id();
    let command = EditorLaunchCommand::new("zircon_editor.exe", request)
        .unwrap()
        .with_hub_handshake(token);

    assert_eq!(
        command.args[2..]
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec![
            "--hub-session",
            "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52",
            "--hub-protocol",
            "1",
        ]
    );
    let transmitted = serde_json::from_str::<ProjectLaunchIntent>(&command.args[1]).unwrap();
    assert_eq!(transmitted.operation_id(), operation_id);
}

#[test]
fn staged_editor_executable_uses_configured_engine_root() {
    let executable = staged_editor_executable("E:/configured/ZirconEngine");

    assert_eq!(
        executable,
        PathBuf::from("E:/configured/ZirconEngine").join(platform_executable_name("zircon_editor"))
    );
}

#[test]
fn staged_engine_launch_always_uses_the_configured_staged_artifact() {
    let command = EditorLaunchCommand::from_staged_engine(
        "E:/configured/ZirconEngine",
        EditorLaunchRequest::open_project("E:/Projects/Game").unwrap(),
    )
    .expect("staged engine launch command should be constructible");

    assert_eq!(
        command.executable,
        PathBuf::from("E:/configured/ZirconEngine").join(platform_executable_name("zircon_editor"))
    );
}

#[test]
fn staged_editor_executable_reports_missing_configured_artifact() {
    let missing =
        std::env::temp_dir().join(format!("zircon_hub_missing_editor_{}", std::process::id()));

    assert!(!staged_editor_executable_exists(missing));
}
