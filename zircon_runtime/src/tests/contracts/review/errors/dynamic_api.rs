//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
#[test]
fn review_f5_dynamic_api_session_uses_typed_errors_before_abi_status_boundary() {
    let session = include_str!("../../../../dynamic_api/session.rs");
    let construction = include_str!("../../../../dynamic_api/session/construction.rs");
    let ffi = include_str!("../../../../dynamic_api/session/ffi.rs");
    let state = include_str!("../../../../dynamic_api/session/state.rs");
    let error = include_str!("../../../../dynamic_api/session/error.rs");
    let project = include_str!("../../../../dynamic_api/session/project.rs");
    let status = include_str!("../../../../dynamic_api/session/status.rs");
    let frame = include_str!("../../../../dynamic_api/frame.rs");
    let bounded_error = include_str!("../../../../dynamic_api/bounded_json/error.rs");
    let _session_doc =
        include_str!("../../../../../../docs/crates/zircon_runtime/dynamic_api/session.md");

    for required in [
        "mod error;",
        "pub use error::{RuntimeDynamicSessionError, RuntimeProjectError};",
        ") -> Result<Self, RuntimeConstructionFailure>",
        "fn tick_frame(&mut self) -> RuntimeDynamicSessionResult<()>",
        "RuntimeDynamicSessionError::RenderBridgeStep",
        "RuntimeDynamicSessionError::ProjectStep",
    ] {
        assert!(
            [session, construction, ffi, state]
                .iter()
                .any(|source| source.contains(required)),
            "dynamic API session owners should contain typed-error anchor `{required}`"
        );
    }
    for required in [
        "pub(super) fn encode_accessibility_tree(",
        ") -> Result<Vec<u8>, BoundedJsonError>",
        "bounded_json::encode(",
        "ZR_RUNTIME_ACCESSIBILITY_TREE_OUTPUT_LIMIT_V1",
        "|| accessibility_tree_item_count(snapshot)",
    ] {
        assert!(
            frame.contains(required),
            "bounded accessibility encoder must retain `{required}`"
        );
    }
    assert!(
        ffi.contains("encode_accessibility_tree(&snapshot).map_err(|error| {")
            && ffi.contains("output_payload_status(")
            && ffi.contains("runtime accessibility tree output exceeds limit"),
        "accessibility encoder errors must reach the typed output-payload ABI boundary"
    );
    for required in [
        "enum BoundedJsonError",
        "EncodedBytes { observed: usize, limit: usize }",
        "Items { observed: usize, limit: usize }",
        "NestingDepth { observed: usize, limit: usize }",
        "ProcessingTime { limit_micros: u64 }",
        "const fn is_limit_exceeded(&self) -> bool",
    ] {
        assert!(
            bounded_error.contains(required),
            "bounded output must preserve error category `{required}`"
        );
    }
    let construction_failure =
        include_str!("../../../../dynamic_api/session/construction/cleanup.rs");
    for required in [
        "struct RuntimeConstructionFailure",
        "primary: RuntimeDynamicSessionError",
        "impl From<RuntimeDynamicSessionError> for RuntimeConstructionFailure",
        "fn from(primary: RuntimeDynamicSessionError) -> Self",
        "impl std::error::Error for RuntimeConstructionFailure",
        "fn source(&self) -> Option<&(dyn std::error::Error + 'static)>",
        "Some(&self.primary)",
    ] {
        assert!(
            construction_failure.contains(required),
            "typed construction failure must preserve {required}"
        );
    }
    assert!(construction.contains(") -> Result<RuntimeDynamicSession, RuntimeConstructionFailure>"));
    for required in [
        "pub(super) type RuntimeDynamicSessionResult<T>",
        "pub(super) type RuntimeProjectResult<T>",
        "pub enum RuntimeDynamicSessionError",
        "pub enum RuntimeProjectError",
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
        "use std::fmt::{self, Display, Write as _};",
        "fn error_status(message: impl Display) -> ZrStatus",
        "fn output_payload_status(",
        "error: BoundedJsonError",
        "if error.is_limit_exceeded()",
        "limit_exceeded(limit_message)",
        "error_status(error)",
        "static STATUS_DIAGNOSTICS: RefCell<StatusDiagnosticsBuffer>",
        "bytes: [u8; ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1]",
        "let available = self.bytes.len().saturating_sub(self.len);",
        "while count > 0 && !value.is_char_boundary(count)",
        "storage.write_fmt(format_args!(\"{message}\"))",
    ] {
        assert!(
            status.contains(required),
            "dynamic API ABI status boundary should contain `{required}`"
        );
    }

    let session_production = [session, construction, ffi, state].concat();
    let project_production = project
        .split("#[cfg(test)]")
        .next()
        .expect("dynamic API project production source");
    for (label, source) in [
        ("dynamic API session", session_production.as_str()),
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
}
