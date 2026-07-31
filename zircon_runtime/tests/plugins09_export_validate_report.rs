use zircon_runtime::asset::{AssetUri, ProjectManifest};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportPackagingStrategy, ExportProfile, ExportTargetPlatform, RuntimeProfileId,
};
use zircon_runtime::plugin::{ExportBuildPlan, ExportValidateReport};

fn source_template_manifest(name: &str) -> ProjectManifest {
    let mut manifest = ProjectManifest::new(
        name,
        AssetUri::parse("res://scenes/main.zscene").expect("scene URI should be valid"),
        1,
    );
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
        RuntimeProfileId::Client2d,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)];
    manifest
}

#[test]
fn export_validate_closeout_uses_compact_generated_file_metadata() {
    let manifest = source_template_manifest("Compact Export Validate Report");
    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client")
        .expect("source-template export plan should build");
    let source_file = plan
        .generated_files
        .iter()
        .find(|file| file.path == "src/main.rs")
        .expect("source template plan should generate main.rs");
    let report = ExportValidateReport::from_build_plan("zircon-project.toml", None, &plan);
    let generated_file = report
        .plan_summary
        .as_ref()
        .expect("validate report should include a plan summary")
        .generated_files
        .iter()
        .find(|file| file.path == "src/main.rs")
        .expect("compact summary should retain generated file metadata");

    assert_eq!(report.schema_version, 2);
    assert_eq!(
        generated_file.byte_length,
        source_file.contents.len() as u64
    );
    assert_eq!(
        generated_file.content_digest,
        ExportValidateReport::sha256_digest(source_file.contents.as_bytes())
    );
    let report_json = serde_json::to_value(&report).expect("report should serialize");
    let serialized_file = report_json["plan_summary"]["generated_files"]
        .as_array()
        .expect("plan summary should serialize generated files")
        .iter()
        .find(|file| file["path"] == source_file.path)
        .expect("compact report JSON should retain the generated file row");
    assert!(serialized_file.get("contents").is_none());
    assert_eq!(
        serialized_file["byte_length"],
        source_file.contents.len() as u64
    );

    let artifact = ExportValidateReport::generated_contents_artifact_json(&plan, false)
        .expect("contents artifact should serialize");
    let artifact_json = serde_json::from_str::<serde_json::Value>(&artifact)
        .expect("contents artifact should be JSON");
    assert_eq!(artifact_json["schema_version"], 1);
    assert!(artifact_json["generated_files"]
        .as_array()
        .expect("contents artifact should serialize generated files")
        .iter()
        .any(|file| file["path"] == source_file.path && file["contents"] == source_file.contents));
}

#[test]
fn export_validate_closeout_excludes_1_and_100_mib_generated_contents_by_default() {
    let manifest = source_template_manifest("Compact Export Validate Report Payload");

    for mebibytes in [1, 100] {
        let mut plan = ExportBuildPlan::from_project_manifest(&manifest, "client")
            .expect("source-template export plan should build");
        let generated_contents = format!(
            "compact-report-payload-marker:{mebibytes}:{}",
            "x".repeat(mebibytes * 1024 * 1024)
        );
        plan.generated_files
            .iter_mut()
            .find(|file| file.path == "src/main.rs")
            .expect("source template plan should generate main.rs")
            .contents = generated_contents;
        let generated_contents = &plan
            .generated_files
            .iter()
            .find(|file| file.path == "src/main.rs")
            .expect("source template plan should retain main.rs")
            .contents;

        let report = ExportValidateReport::from_build_plan("zircon-project.toml", None, &plan);
        let report_json = serde_json::to_string(&report).expect("report should serialize");
        assert!(!report_json.contains(generated_contents));
        assert!(
            report_json.len() < generated_contents.len() / 10,
            "default report must remain compact at {mebibytes} MiB"
        );

        let artifact = ExportValidateReport::generated_contents_artifact_json(&plan, false)
            .expect("explicit contents artifact should serialize");
        let artifact_digest = ExportValidateReport::sha256_digest(artifact.as_bytes());
        let mut report_with_artifact = report;
        report_with_artifact.record_generated_contents_artifact(
            "out/generated-contents.json".to_string(),
            artifact.len() as u64,
            artifact_digest.clone(),
        );
        let report_with_artifact_json =
            serde_json::to_value(&report_with_artifact).expect("artifact report should serialize");

        assert!(artifact.contains(generated_contents));
        assert_eq!(
            report_with_artifact_json["generated_contents_artifact_path"],
            "out/generated-contents.json"
        );
        assert_eq!(
            report_with_artifact_json["generated_contents_artifact_byte_length"],
            artifact.len() as u64
        );
        assert_eq!(
            report_with_artifact_json["generated_contents_artifact_digest"],
            artifact_digest
        );
    }
}
