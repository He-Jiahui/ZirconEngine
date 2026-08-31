use std::path::PathBuf;

use uuid::Uuid;

use super::super::{
    ProjectActivationOperationId, ProjectActivationOperationSequence, ProjectLaunchInstanceId,
    ProjectLaunchIntent, ProjectLaunchIntentError, ProjectLaunchProfile, ProjectLaunchSource,
    ProjectLaunchTarget, ProjectTemplateId, PROJECT_LAUNCH_INTENT_SCHEMA_VERSION_V1,
};

fn operation_id() -> ProjectActivationOperationId {
    ProjectActivationOperationId::try_from_parts(
        ProjectLaunchInstanceId::try_from_uuid(
            Uuid::parse_str("4ed7a2f4-fd6a-4e12-9506-44b17e190213").unwrap(),
        )
        .unwrap(),
        ProjectActivationOperationSequence::new(17).unwrap(),
        Uuid::parse_str("a32e5f32-d9d9-47a9-a423-55635d418a58").unwrap(),
    )
    .unwrap()
}

#[test]
fn project_launch_intent_preserves_versioned_operation_and_untrusted_path_input() {
    let intent = ProjectLaunchIntent::open_existing(
        operation_id(),
        ProjectLaunchSource::Hub,
        ProjectLaunchProfile::Safe,
        "E:/Projects/My Game",
    )
    .unwrap();

    assert_eq!(
        intent.schema_version(),
        PROJECT_LAUNCH_INTENT_SCHEMA_VERSION_V1
    );
    assert_eq!(intent.operation_id(), operation_id());
    assert_eq!(intent.source(), ProjectLaunchSource::Hub);
    assert_eq!(intent.profile(), ProjectLaunchProfile::Safe);
    assert_eq!(
        intent.target(),
        &ProjectLaunchTarget::OpenExisting {
            requested_path: PathBuf::from("E:/Projects/My Game"),
        }
    );
    assert_eq!(
        serde_json::from_str::<ProjectLaunchIntent>(&serde_json::to_string(&intent).unwrap())
            .unwrap(),
        intent
    );
}

#[test]
fn project_creation_retargets_the_same_operation_after_the_create_transaction() {
    let intent = ProjectLaunchIntent::create_project(
        operation_id(),
        ProjectLaunchSource::Welcome,
        ProjectLaunchProfile::Normal,
        "My Game",
        "E:/Projects",
        ProjectTemplateId::RenderableEmpty,
    )
    .unwrap();

    let opened = intent
        .retarget_open_existing_project("E:/Projects/My Game")
        .unwrap();

    assert_eq!(opened.operation_id(), intent.operation_id());
    assert_eq!(opened.source(), ProjectLaunchSource::Welcome);
    assert_eq!(
        opened.target(),
        &ProjectLaunchTarget::OpenExisting {
            requested_path: PathBuf::from("E:/Projects/My Game"),
        }
    );
}

#[test]
fn project_launch_intent_rejects_malformed_or_unknown_wire_inputs() {
    assert_eq!(
        ProjectLaunchIntent::open_existing(
            operation_id(),
            ProjectLaunchSource::Cli,
            ProjectLaunchProfile::Normal,
            " ",
        ),
        Err(ProjectLaunchIntentError::EmptyOpenPath)
    );

    let unsupported_schema = serde_json::json!({
        "schema_version": 2,
        "operation_id": operation_id(),
        "source": "cli",
        "profile": "normal",
        "target": { "kind": "open_existing", "requested_path": "E:/Projects/Game" },
    });
    assert!(serde_json::from_value::<ProjectLaunchIntent>(unsupported_schema).is_err());

    let mut unknown_field = serde_json::to_value(
        ProjectLaunchIntent::create_project(
            operation_id(),
            ProjectLaunchSource::Recent,
            ProjectLaunchProfile::Recovery,
            "Game",
            "E:/Projects",
            ProjectTemplateId::RenderableEmpty,
        )
        .unwrap(),
    )
    .unwrap();
    unknown_field
        .as_object_mut()
        .unwrap()
        .insert("forged_identity".to_string(), serde_json::Value::Null);
    assert!(serde_json::from_value::<ProjectLaunchIntent>(unknown_field).is_err());
}
