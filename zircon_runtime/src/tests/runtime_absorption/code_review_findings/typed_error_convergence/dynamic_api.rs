#[test]
fn review_f5_dynamic_api_session_uses_typed_errors_before_abi_status_boundary() {
    let session = include_str!("../../../../dynamic_api/session.rs");
    let error = include_str!("../../../../dynamic_api/session/error.rs");
    let project = include_str!("../../../../dynamic_api/session/project.rs");
    let status = include_str!("../../../../dynamic_api/session/status.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let session_doc = include_str!("../../../../../../docs/zircon_runtime/dynamic_api/session.md");
    let status_rows = include_str!(
        "../../plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs"
    );
    let status_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

    for required in [
        "mod error;",
        "use error::{RuntimeDynamicSessionError, RuntimeDynamicSessionResult};",
        ") -> RuntimeDynamicSessionResult<Self>",
        "fn tick_frame(&mut self) -> RuntimeDynamicSessionResult<()>",
        "RuntimeDynamicSessionError::RenderBridgeStep",
        "RuntimeDynamicSessionError::ProjectStep",
        "RuntimeDynamicSessionError::EncodeAccessibilityTree",
    ] {
        assert!(
            session.contains(required),
            "dynamic API session should contain typed-error anchor `{required}`"
        );
    }
    for required in [
        "pub(super) type RuntimeDynamicSessionResult<T>",
        "pub(super) type RuntimeProjectResult<T>",
        "pub(super) enum RuntimeDynamicSessionError",
        "pub(super) enum RuntimeProjectError",
        "ModuleDiscovery",
        "CoreStep",
        "ProjectStep",
        "RenderBridgeStep",
        "RuntimeExtensionRegistryStep",
        "ProjectRootUtf8",
        "LoadScriptPackage",
        "MissingStartupScriptPackage",
    ] {
        assert!(
            error.contains(required),
            "dynamic API error owner should contain `{required}`"
        );
    }
    for required in [
        "RuntimeProjectResult<Option<Self>>",
        "RuntimeProjectError::ProjectRootUtf8",
        "RuntimeProjectError::OpenProject",
        "RuntimeProjectError::ResolveAssetManager",
        "RuntimeProjectError::LoadDefaultScene",
        "RuntimeProjectError::LoadNavmesh",
        "RuntimeProjectError::DiscoverScriptPackages",
        "RuntimeProjectError::LoadScriptPackage",
    ] {
        assert!(
            project.contains(required),
            "dynamic API project owner should contain typed project error anchor `{required}`"
        );
    }
    for required in [
        "use std::fmt::Display;",
        "fn error_status(message: impl Display) -> ZrStatus",
        "let message = message.to_string();",
    ] {
        assert!(
            status.contains(required),
            "dynamic API ABI status boundary should contain `{required}`"
        );
    }

    let session_production = session
        .split("#[cfg(test)]")
        .next()
        .expect("dynamic API session production source");
    let project_production = project
        .split("#[cfg(test)]")
        .next()
        .expect("dynamic API project production source");
    for (label, source) in [
        ("dynamic API session", session_production),
        ("dynamic API project", project_production),
    ] {
        for forbidden in [
            "Result<Self, String>",
            "Result<(), String>",
            ".map_err(|error| error.to_string())",
            ".map_err(|error| format!",
            "Err(format!",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep lossy String error transport `{forbidden}`"
            );
        }
    }

    for doc_anchor in [
        "Runtime 15 F5 dynamic API session typed errors",
        "runtime_15_dynamic_api_session_typed_errors_static_passed_cargo_deferred",
        "review_f5_dynamic_api_session_uses_typed_errors_before_abi_status_boundary",
        "dynamic_api/session/error.rs",
        "RuntimeDynamicSessionError::RenderBridgeStep",
        "RuntimeProjectError::LoadDefaultScene",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || session_doc.contains(doc_anchor)
                || status_rows.contains(doc_anchor)
                || status_map.contains(doc_anchor)
                || date_map.contains(doc_anchor),
            "F5 dynamic API typed-error docs/status should record `{doc_anchor}`"
        );
    }
}
