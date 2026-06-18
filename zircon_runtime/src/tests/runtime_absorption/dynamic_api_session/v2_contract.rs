use std::fs;
use std::path::Path;

const RUNTIME_10_UI_V2_CONTRACT_STATUS: &str =
    "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending";
const RUNTIME_09_V2_VERDICT_STATUS: &str =
    "runtime_09_v2_verdict_matches_runtime_and_interface_modules";

#[test]
fn runtime_10_ui_v2_contract_sync_matches_runtime_09_verdict_and_interface_owner() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = runtime_root
        .parent()
        .expect("runtime crate should live under the repository root");

    assert_runtime_09_v2_verdict_is_documented(repo_root);
    assert_interface_owns_ui_v2_contracts(repo_root);
    assert_runtime_consumes_interface_ui_v2_contracts(repo_root);
    assert_named_api_version_mismatch_guard_exists(repo_root);
    assert_runtime_10_docs_record_ui_v2_contract_sync(repo_root);
}

fn assert_runtime_09_v2_verdict_is_documented(repo_root: &Path) {
    for relative_doc in [
        "docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md",
        "docs/zircon_runtime/ui/architecture.md",
        "docs/plans/zircon_runtime/runtime/index.md",
    ] {
        let doc = read_repo_file(repo_root, relative_doc);
        for required_anchor in [
            "v2-replacement-mainline",
            RUNTIME_09_V2_VERDICT_STATUS,
            ".zui",
            ".v2.ui.toml",
        ] {
            assert!(
                doc.contains(required_anchor),
                "`{relative_doc}` should keep Runtime 09 v2 verdict anchor `{required_anchor}`"
            );
        }
    }
}

fn assert_interface_owns_ui_v2_contracts(repo_root: &Path) {
    let interface_v2_mod = read_repo_file(repo_root, "zircon_runtime_interface/src/ui/v2/mod.rs");
    for required_export in [
        "pub use asset::{",
        "UiV2AssetDocument",
        "UiV2CompiledDocument",
        "UiV2ComponentGraph",
        "UiV2StyleSheet",
    ] {
        assert!(
            interface_v2_mod.contains(required_export),
            "interface ui/v2 should keep public DTO export `{required_export}`"
        );
    }

    let api_version = read_repo_file(
        repo_root,
        "zircon_runtime_interface/src/ui/template/asset/component_contract/api_version.rs",
    );
    for required_anchor in [
        "pub struct UiComponentApiVersion",
        "pub const fn is_compatible_with(self, required: Self) -> bool",
        "type Err = UiComponentApiVersionParseError",
        "invalid ui component api version",
    ] {
        assert!(
            api_version.contains(required_anchor),
            "UiComponentApiVersion owner should keep anchor `{required_anchor}`"
        );
    }
}

fn assert_runtime_consumes_interface_ui_v2_contracts(repo_root: &Path) {
    let runtime_v2_mod = read_repo_file(repo_root, "zircon_runtime/src/ui/v2/mod.rs");
    assert!(
        runtime_v2_mod.contains("pub use zircon_runtime_interface::ui::v2::UiV2CompiledDocument;"),
        "runtime ui/v2 should re-export only the interface-owned compiled document contract"
    );

    for relative_file in [
        "zircon_runtime/src/ui/v2/cache.rs",
        "zircon_runtime/src/ui/v2/compiler.rs",
        "zircon_runtime/src/ui/v2/component_instancer.rs",
        "zircon_runtime/src/ui/v2/file_cache.rs",
        "zircon_runtime/src/ui/v2/loader.rs",
    ] {
        let source = read_repo_file(repo_root, relative_file);
        assert!(
            source.contains("zircon_runtime_interface::ui::v2"),
            "`{relative_file}` should consume interface-owned ui/v2 DTOs"
        );
    }

    assert!(
        !runtime_ui_tree_contains(repo_root, "pub struct UiComponentApiVersion")
            && !runtime_ui_tree_contains(repo_root, "pub enum UiComponentApiVersion"),
        "runtime ui tree must not redefine interface-owned UiComponentApiVersion"
    );

    let validation = read_repo_file(
        repo_root,
        "zircon_runtime/src/ui/template/asset/component_contract/validation.rs",
    );
    for required_anchor in [
        "UiComponentApiVersion",
        "actual.is_compatible_with(required)",
        "UiComponentContractDiagnosticCode::ApiMismatch",
    ] {
        assert!(
            validation.contains(required_anchor),
            "runtime component contract validation should keep interface API-version anchor `{required_anchor}`"
        );
    }
}

fn assert_named_api_version_mismatch_guard_exists(repo_root: &Path) {
    let interface_guard = read_repo_file(
        repo_root,
        "zircon_runtime_interface/src/tests/ui_v2_contracts.rs",
    );
    assert!(
        interface_guard.contains("ui_component_api_version_mismatch_is_rejected_with_parse_error"),
        "interface tests should keep the named UiComponentApiVersion mismatch guard requested by Runtime 10 M2.2"
    );

    let runtime_contract_tests = read_repo_file(
        repo_root,
        "zircon_runtime/src/ui/tests/asset_component_contract.rs",
    );
    for required_anchor in [
        "component_contract_rejects_incompatible_imported_component_api_version",
        "component_contract_reports_api_mismatch_diagnostic_target_node",
        "component_contract_rejects_invalid_api_version_strings",
    ] {
        assert!(
            runtime_contract_tests.contains(required_anchor),
            "runtime UI component contract tests should keep behavior anchor `{required_anchor}`"
        );
    }
}

fn assert_runtime_10_docs_record_ui_v2_contract_sync(repo_root: &Path) {
    for relative_doc in [
        "docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md",
        "docs/plans/zircon_runtime/runtime/index.md",
        "docs/engine-architecture/runtime-interface-convergence.md",
        "docs/engine-architecture/runtime-architecture-review-m0.md",
        "docs/zircon_runtime/dynamic_api/session.md",
        "docs/zircon_runtime_interface/ui/mod.md",
        "docs/zircon_runtime/ui/architecture.md",
    ] {
        let doc = read_repo_file(repo_root, relative_doc);
        for required_anchor in [
            RUNTIME_10_UI_V2_CONTRACT_STATUS,
            "ui_v2_contract_sync_anchors = 9/9",
            "UiComponentApiVersion",
            "v2-replacement-mainline",
        ] {
            assert!(
                doc.contains(required_anchor),
                "`{relative_doc}` should mirror Runtime 10 M2.2 UI v2 contract anchor `{required_anchor}`"
            );
        }
    }
}

fn runtime_ui_tree_contains(repo_root: &Path, needle: &str) -> bool {
    source_tree_contains(&repo_root.join("zircon_runtime/src/ui"), needle)
}

fn source_tree_contains(root: &Path, needle: &str) -> bool {
    let Ok(entries) = fs::read_dir(root) else {
        return false;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            if source_tree_contains(&path, needle) {
                return true;
            }
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        if fs::read_to_string(&path)
            .map(|source| source.contains(needle))
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

fn read_repo_file(repo_root: &Path, relative_file: &str) -> String {
    fs::read_to_string(repo_root.join(relative_file))
        .unwrap_or_else(|error| panic!("`{relative_file}` should be readable: {error}"))
}
