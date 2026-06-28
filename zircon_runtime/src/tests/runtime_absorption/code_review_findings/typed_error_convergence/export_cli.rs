#[test]
fn review_f5_export_cli_uses_typed_errors_before_cli_boundary() {
    let pack_args = include_str!("../../../../bin/zircon_export_pack/args.rs");
    let pack_error = include_str!("../../../../bin/zircon_export_pack/error.rs");
    let pack_main = include_str!("../../../../bin/zircon_export_pack/main.rs");
    let pack_manifest = include_str!("../../../../bin/zircon_export_pack/manifest.rs");
    let pack_run = include_str!("../../../../bin/zircon_export_pack/run.rs");
    let validate_args = include_str!("../../../../bin/zircon_export_validate/args.rs");
    let validate_error = include_str!("../../../../bin/zircon_export_validate/error.rs");
    let validate_main = include_str!("../../../../bin/zircon_export_validate/main.rs");
    let validate_run = include_str!("../../../../bin/zircon_export_validate/run.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let export_tool_doc =
        include_str!("../../../../../../docs/cli-and-tooling/zircon-export-tool.md");
    let pack_doc = include_str!("../../../../../../docs/zircon_runtime/asset/pack.md");
    let validate_doc =
        include_str!("../../../../../../docs/zircon_runtime/plugin/export_build_plan.md");

    for required in [
        "mod error;",
        "pub type ExportPackResult<T> = std::result::Result<T, ExportPackError>;",
        "pub enum ExportPackError",
        "Usage(String)",
        "ReadAssetManifest {",
        "DecodeAssetManifest {",
        "ReadPreviousZrPack {",
        "VerifyDeltaAsset {",
        "DeltaApplyVerificationMismatch",
        "DeterministicComparisonWrite {",
        "#[source]",
        "source: io::Error",
        "source: serde_json::Error",
        "source: ZrPackError",
    ] {
        assert!(
            pack_main.contains(required) || pack_error.contains(required),
            "zircon_export_pack typed error boundary should contain `{required}`"
        );
    }

    for required in [
        "pub fn parse(args: impl IntoIterator<Item = OsString>) -> ExportPackResult<Option<PackArgs>>",
        "ExportPackError::Usage(usage(",
        ") -> ExportPackResult<String>",
        ") -> ExportPackResult<PathBuf>",
        "pub fn pack_inputs(&self, manifest_dir: &Path) -> ExportPackInputs",
        "pub fn run(args: impl IntoIterator<Item = OsString>) -> ExportPackResult<ExitCode>",
        ") -> ExportPackResult<Option<VerifiedDeltaWriteReport>>",
        ") -> ExportPackResult<bool>",
        "ExportPackError::ReadAssetManifest",
        "ExportPackError::CreatePackDirectory",
        "ExportPackError::WriteDeltaPack",
        "ExportPackError::DeterministicComparisonWrite",
    ] {
        assert!(
            pack_args.contains(required)
                || pack_manifest.contains(required)
                || pack_run.contains(required),
            "zircon_export_pack typed execution path should contain `{required}`"
        );
    }

    for (label, source) in [
        ("pack args", pack_args),
        ("pack manifest", pack_manifest),
        ("pack run", pack_run),
    ] {
        for forbidden in [
            "Result<Option<PackArgs>, String>",
            "Result<ExportPackInputs, String>",
            "Result<ExitCode, String>",
            "Result<Option<VerifiedDeltaWriteReport>, String>",
            "Result<bool, String>",
            "Err(format!(",
            "return Err(\"delta pack apply verification did not reconstruct target zrpack\"",
            ".map_err(|error| format!(",
            ".map_err(|error| error.to_string())",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep lossy String error branch `{forbidden}`"
            );
        }
    }

    for required in [
        "mod error;",
        "pub type ExportValidateResult<T> = std::result::Result<T, ExportValidateError>;",
        "pub enum ExportValidateError",
        "Usage(String)",
        "EncodeReport {",
        "CreateReportDirectory {",
        "WriteReport {",
        "#[source]",
        "source: serde_json::Error",
        "source: io::Error",
    ] {
        assert!(
            validate_main.contains(required) || validate_error.contains(required),
            "zircon_export_validate typed error boundary should contain `{required}`"
        );
    }

    for required in [
        ") -> ExportValidateResult<Option<ValidateArgs>>",
        "ExportValidateError::Usage(usage(",
        ") -> ExportValidateResult<String>",
        ") -> ExportValidateResult<PathBuf>",
        "pub fn run(args: impl IntoIterator<Item = OsString>) -> ExportValidateResult<ExitCode>",
        "ExportValidateError::EncodeReport",
        "ExportValidateError::CreateReportDirectory",
        "ExportValidateError::WriteReport",
    ] {
        assert!(
            validate_args.contains(required) || validate_run.contains(required),
            "zircon_export_validate typed execution path should contain `{required}`"
        );
    }

    for (label, source) in [
        ("validate args", validate_args),
        ("validate run", validate_run),
    ] {
        for forbidden in [
            "Result<Option<ValidateArgs>, String>",
            "Result<ExitCode, String>",
            "Err(format!(",
            ".map_err(|error| format!(",
            ".map_err(|error| error.to_string())",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep lossy String error branch `{forbidden}`"
            );
        }
    }

    assert!(
        pack_main.contains("eprintln!(\"{error}\");")
            && validate_main.contains("eprintln!(\"{error}\");"),
        "main.rs should remain the final CLI display boundary for typed export errors"
    );

    for doc_anchor in [
        "Runtime 15 F5 export CLI typed errors",
        "runtime_15_export_cli_typed_errors_static_passed_cargo_deferred",
        "review_f5_export_cli_uses_typed_errors_before_cli_boundary",
        "zircon_export_pack/error.rs",
        "zircon_export_validate/error.rs",
        "ExportPackError::ReadAssetManifest",
        "ExportValidateError::EncodeReport",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || export_tool_doc.contains(doc_anchor)
                || pack_doc.contains(doc_anchor)
                || validate_doc.contains(doc_anchor),
            "export CLI typed error docs should record `{doc_anchor}`"
        );
    }
}
