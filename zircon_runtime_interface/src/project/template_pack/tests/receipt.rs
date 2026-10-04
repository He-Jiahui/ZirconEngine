use uuid::Uuid;

use crate::project::{
    project_template_descriptor, ProjectActivationOperationId, ProjectActivationOperationSequence,
    ProjectLaunchInstanceId, ProjectTemplateId,
};
use crate::runtime_build_set::ZrRuntimeBuildSetId;

use super::{
    ProjectCreationProvenance, ProjectTemplateReceipt, PROJECT_TEMPLATE_RECEIPT_SCHEMA_VERSION_V1,
};

fn receipt() -> ProjectTemplateReceipt {
    let operation_id = ProjectActivationOperationId::try_from_parts(
        ProjectLaunchInstanceId::new(),
        ProjectActivationOperationSequence::new(1).expect("non-zero sequence"),
        Uuid::new_v4(),
    )
    .expect("operation id");
    ProjectTemplateReceipt::try_new(
        project_template_descriptor(ProjectTemplateId::RenderableEmpty),
        crate::project::ProjectGuid::new(),
        operation_id,
        crate::project::ProjectEngineVersion::parse("0.1.4").expect("engine version"),
        ZrRuntimeBuildSetId::parse(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .expect("build set id"),
    )
    .expect("compatible template receipt")
}

#[test]
fn creation_provenance_issues_the_exact_authenticated_identity() {
    let expected = receipt();
    let provenance = ProjectCreationProvenance::new(
        expected.operation_id(),
        expected.creator_engine_version().clone(),
        expected.build_set_id().clone(),
    );

    let actual = provenance
        .issue_receipt(*expected.descriptor(), expected.project_guid())
        .expect("issue receipt from compatible provenance");

    assert_eq!(actual, expected);
}

#[test]
fn receipt_round_trips_with_all_provenance_identity() {
    let expected = receipt();
    let encoded = serde_json::to_string(&expected).expect("serialize receipt");
    let decoded: ProjectTemplateReceipt =
        serde_json::from_str(&encoded).expect("deserialize receipt");

    assert_eq!(decoded, expected);
    assert_eq!(
        decoded.schema_version(),
        PROJECT_TEMPLATE_RECEIPT_SCHEMA_VERSION_V1
    );
    assert_eq!(
        decoded.descriptor().qualified_id(),
        "zircon/renderable-empty@1"
    );
    assert_eq!(decoded.creator_engine_version().to_string(), "0.1.4");
}

#[test]
fn receipt_rejects_unknown_schema_and_fields() {
    let encoded = serde_json::to_value(receipt()).expect("serialize receipt");

    let mut wrong_version = encoded.clone();
    wrong_version["schema_version"] = serde_json::json!(2);
    assert!(serde_json::from_value::<ProjectTemplateReceipt>(wrong_version).is_err());

    let mut unknown_field = encoded;
    unknown_field["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ProjectTemplateReceipt>(unknown_field).is_err());
}

#[test]
fn receipt_rejects_an_incompatible_creator_engine() {
    let mut encoded = serde_json::to_value(receipt()).expect("serialize receipt");
    encoded["creator_engine_version"] = serde_json::json!("0.2.0");

    assert!(serde_json::from_value::<ProjectTemplateReceipt>(encoded).is_err());
}
