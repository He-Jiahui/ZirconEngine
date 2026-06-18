use crate::ui::template::{
    UiRuntimeCompiledAssetArtifact, UiTemplateRuntimePipeline, UiTemplateRuntimePipelineError,
};
use zircon_runtime_interface::ui::{event_ui::UiTreeId, template::UiTemplateError};

#[test]
fn template_validate_rejects_unknown_component_contract() {
    let error = UiTemplateRuntimePipeline::build_surface_from_toml_str(
        UiTreeId::new("template.unknown_component"),
        r#"
version = 1

[root]
template = "MissingComponent"
"#,
    )
    .expect_err("unknown component references should fail during template validation");

    assert_eq!(
        error,
        UiTemplateRuntimePipelineError::Validate(UiTemplateError::UnknownTemplate {
            template_id: "MissingComponent".to_string(),
        })
    );
}

#[test]
fn template_instance_failure_surfaces_loader_error() {
    let error = UiTemplateRuntimePipeline::build_surface_from_toml_str(
        UiTreeId::new("template.loader_error"),
        "version = 1\n[root\ncomponent = \"Panel\"\n",
    )
    .expect_err("invalid TOML should fail before validation or instantiation");

    assert!(matches!(
        error,
        UiTemplateRuntimePipelineError::Load(UiTemplateError::ParseToml(_))
    ));
}

#[test]
fn compiled_template_artifact_stays_binary_leaf_dto_not_generated_source() {
    assert_eq!(
        UiRuntimeCompiledAssetArtifact::generated_policy(),
        "runtime_09_m3_1_binary_leaf_dto_artifact_not_generated_source"
    );
    assert!(!UiRuntimeCompiledAssetArtifact::requires_generated_source_marker());
}
