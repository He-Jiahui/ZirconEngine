#[test]
fn review_f5_profile_export_uses_typed_error() {
    let export = include_str!("../../../../core/runtime/diagnostics/profiling/export.rs");
    let profiling_mod = include_str!("../../../../core/runtime/diagnostics/profiling/mod.rs");
    let diagnostics_mod = include_str!("../../../../core/runtime/diagnostics/mod.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let profiling_doc =
        include_str!("../../../../../../docs/zircon_runtime/core/diagnostics/profiling.md");

    for required in [
        "pub type ProfileExportResult<T> = std::result::Result<T, ProfileExportError>;",
        "pub enum ProfileExportError",
        "FeatureDisabled",
        "CreateExportDirectory",
        "JsonSerialize",
        "WriteFile",
        ") -> ProfileExportResult<ProfileExportReport>",
        "ProfileExportError::CreateExportDirectory",
        "ProfileExportError::WriteFile",
    ] {
        assert!(
            export.contains(required),
            "F5 profile export typed-error owner should contain `{required}`"
        );
    }
    for required in [
        "pub use export::{ProfileExportError, ProfileExportReport, ProfileExportResult};",
        "pub fn export_report() -> ProfileExportResult<ProfileExportReport>",
        "ProfileExportError::FeatureDisabled",
        "pub fn stop_and_export_capture_from_env() -> Option<ProfileExportResult<ProfileExportReport>>",
        "ProfileControlResponse::error(error.to_string())",
    ] {
        assert!(
            profiling_mod.contains(required),
            "F5 profile module boundary should contain `{required}`"
        );
    }
    for required in ["ProfileExportError", "ProfileExportResult"] {
        assert!(
            diagnostics_mod.contains(required),
            "runtime diagnostics facade should re-export `{required}`"
        );
    }
    for (label, source) in [
        ("profiling export", export),
        ("profiling module", profiling_mod),
    ] {
        for forbidden in [
            "Result<ProfileExportReport, String>",
            "Result<(), String>",
            ".map_err(|error| error.to_string())",
            "\"profiling feature is disabled\".to_string()",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep lossy String profile export branch `{forbidden}`"
            );
        }
    }
}
