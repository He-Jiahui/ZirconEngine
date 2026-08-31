use std::str::FromStr;

use uuid::Uuid;

use super::super::{
    render_project_template, ProjectGuid, ProjectManifestSummary, ProjectTemplateId,
};

#[test]
fn project_guid_preserves_a_canonical_uuid_across_parse_and_display() {
    let source = "4fa5e9c1-1a6f-497d-b7a3-29f0e4f6cb14";

    let guid = ProjectGuid::from_str(source).unwrap();

    assert_eq!(guid.to_string(), source);
    assert_eq!(guid.as_uuid(), Uuid::parse_str(source).unwrap());
}

#[test]
fn project_guid_rejects_malformed_text() {
    assert!(ProjectGuid::from_str("not-a-project-guid").is_err());
}

#[test]
fn project_guid_rejects_the_nil_uuid_identity() {
    assert!(ProjectGuid::from_str("00000000-0000-0000-0000-000000000000").is_err());
    assert!(
        serde_json::from_str::<ProjectGuid>("\"00000000-0000-0000-0000-000000000000\"").is_err()
    );
}

#[test]
fn generated_project_guids_are_non_nil_and_distinct() {
    let first = ProjectGuid::new();
    let second = ProjectGuid::new();

    assert_ne!(first.as_uuid(), Uuid::nil());
    assert_ne!(first, second);
}

#[test]
fn rendered_project_templates_persist_distinct_project_guids_in_the_manifest() {
    let first =
        render_project_template(ProjectTemplateId::RenderableEmpty, "First Project").unwrap();
    let second =
        render_project_template(ProjectTemplateId::RenderableEmpty, "Second Project").unwrap();
    let first_guid = first
        .summary
        .project_guid
        .expect("a rendered project must receive a persistent guid");
    let second_guid = second
        .summary
        .project_guid
        .expect("a rendered project must receive a persistent guid");

    assert_ne!(first_guid, second_guid);
    let manifest = first
        .entries
        .iter()
        .find(|entry| entry.path.as_str() == "zircon-project.toml")
        .expect("rendered template manifest");
    let parsed = ProjectManifestSummary::parse_toml_bytes(&manifest.bytes).unwrap();
    assert_eq!(parsed.value.project_guid, Some(first_guid));
}

#[test]
fn current_manifest_summary_rejects_a_missing_project_guid() {
    let error = ProjectManifestSummary::parse_toml_str(
        r#"
name = "Missing Guid"
format_version = 3
default_scene = "res://scenes/main.scene.toml"
library_version = 1
"#,
    )
    .unwrap_err();

    assert!(error.to_string().contains("project_guid"));
}
