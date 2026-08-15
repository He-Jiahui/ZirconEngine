use std::fs;
use std::path::Path;

use super::shared::abi::{
    EXPECTED_RUNTIME_10_FUNCTION_TABLES, EXPECTED_RUNTIME_10_SESSION_OPERATIONS,
};
use super::shared::behavior::EXPECTED_RUNTIME_10_BEHAVIOR_TEST_ANCHORS;
use super::shared::diagnostics::{
    EXPECTED_RUNTIME_10_RUNTIME_DIAGNOSTICS_ANCHORS,
    EXPECTED_RUNTIME_10_SCENE_ASSET_RELOAD_DIAGNOSTIC_PATH_ANCHORS,
};
use super::shared::docs::EXPECTED_RUNTIME_10_MIRROR_DOCS;
use super::shared::host_requests::EXPECTED_RUNTIME_10_HOST_REQUEST_PAYLOAD_ANCHORS;
use super::shared::source_inventory::EXPECTED_RUNTIME_10_SOURCE_FILES;

const EXTRA_RUNTIME_10_GUARD_ANCHOR_FILES: &[&str] = &[];

#[test]
fn runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = runtime_root
        .parent()
        .expect("runtime crate should live under the repository root");
    let exports_source =
        fs::read_to_string(repo_root.join("zircon_runtime/src/dynamic_api/exports.rs"))
            .expect("dynamic API exports source should be readable");
    let session_source =
        fs::read_to_string(repo_root.join("zircon_runtime/src/dynamic_api/session/ffi.rs"))
            .expect("dynamic API session FFI source should be readable");
    let operation_source =
        fs::read_to_string(repo_root.join("zircon_runtime/src/dynamic_api/session/operation.rs"))
            .expect("dynamic API operation source should be readable");

    assert_runtime_10_files_exist(repo_root, EXPECTED_RUNTIME_10_SOURCE_FILES);
    assert_function_table_shapes(repo_root);
    assert_runtime_10_ffi_wrappers(&exports_source, &session_source, &operation_source);
    assert_runtime_10_v6_only_hard_cutover(repo_root);
    assert_runtime_10_behavior_test_anchors(repo_root);
    assert_runtime_10_host_request_payload_anchors(repo_root);

    for required_anchor in [
        "runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces",
        "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
        "runtime_10_dynamic_session_test_owner_split_keeps_focused_modules",
        "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending",
        "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending",
        "runtime_10_ui_v2_contract_sync_matches_runtime_09_verdict_and_interface_owner",
        "runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts",
        "runtime_10_profile_control_exposes_runtime_diagnostics_snapshot_without_abi_table_growth",
    ] {
        assert!(
            source_tree_contains(repo_root, required_anchor),
            "Runtime 10 dynamic API tree should keep guard anchor `{required_anchor}`"
        );
    }

    for relative_doc in EXPECTED_RUNTIME_10_MIRROR_DOCS {
        let doc = fs::read_to_string(repo_root.join(relative_doc))
            .unwrap_or_else(|error| panic!("`{relative_doc}` should be readable: {error}"));
        for required_doc_anchor in [
            "dynamic_runtime_api_boundary",
            "expected_source_file_count = 51",
            "ZrRuntimeApiV6",
            "runtime_session_ffi_wrappers = 22/22",
            "runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc.contains(required_doc_anchor),
                "`{relative_doc}` should mirror Runtime 10 audit anchor `{required_doc_anchor}`"
            );
        }
    }
}

fn assert_runtime_10_files_exist(repo_root: &Path, files: &[&str]) {
    assert_eq!(
        files.len(),
        51,
        "Runtime 10 dynamic API source inventory should stay at 51 files"
    );
    for relative_file in files {
        assert!(
            repo_root.join(relative_file).exists(),
            "Runtime 10 dynamic API source file `{relative_file}` should exist"
        );
    }
}

fn assert_runtime_10_behavior_test_anchors(repo_root: &Path) {
    let mut behavior_anchor_count = 0;
    for (relative_file, expected_anchors) in EXPECTED_RUNTIME_10_BEHAVIOR_TEST_ANCHORS {
        let source = fs::read_to_string(repo_root.join(relative_file))
            .unwrap_or_else(|error| panic!("`{relative_file}` should be readable: {error}"));
        for expected_anchor in *expected_anchors {
            behavior_anchor_count += 1;
            assert!(
                source.contains(expected_anchor),
                "`{relative_file}` should keep Runtime 10 behavior test anchor `{expected_anchor}`"
            );
        }
    }
    assert_eq!(
        behavior_anchor_count, 16,
        "Runtime 10 behavior test anchor inventory should stay at 16 tests"
    );
}

#[test]
fn runtime_10_dynamic_runtime_api_mirror_docs_include_runtime_diagnostics_anchors() {
    assert_eq!(
        EXPECTED_RUNTIME_10_RUNTIME_DIAGNOSTICS_ANCHORS.len(),
        15,
        "Runtime 10 runtime diagnostics profile-control inventory should stay at 15 anchors"
    );
    assert_eq!(
        EXPECTED_RUNTIME_10_SCENE_ASSET_RELOAD_DIAGNOSTIC_PATH_ANCHORS.len(),
        21,
        "Runtime 10 scene-asset reload diagnostic path inventory should stay at 21 anchors"
    );
}

fn assert_runtime_10_host_request_payload_anchors(repo_root: &Path) {
    assert_eq!(
        EXPECTED_RUNTIME_10_HOST_REQUEST_PAYLOAD_ANCHORS.len(),
        38,
        "Runtime 10 host-request payload anchor inventory should stay at 38 anchors"
    );
    for (relative_file, expected_anchor) in EXPECTED_RUNTIME_10_HOST_REQUEST_PAYLOAD_ANCHORS {
        let source = fs::read_to_string(repo_root.join(relative_file))
            .unwrap_or_else(|error| panic!("`{relative_file}` should be readable: {error}"));
        assert!(
            source.contains(expected_anchor),
            "`{relative_file}` should keep Runtime 10 host-request payload anchor `{expected_anchor}`"
        );
    }
}

fn assert_function_table_shapes(repo_root: &Path) {
    assert_eq!(
        EXPECTED_RUNTIME_10_FUNCTION_TABLES.len(),
        10,
        "Runtime 10 ABI inventory should keep 10 function-table structs"
    );
    for (relative_file, table_name, expected_fields) in EXPECTED_RUNTIME_10_FUNCTION_TABLES {
        let source = fs::read_to_string(repo_root.join(relative_file))
            .unwrap_or_else(|error| panic!("`{relative_file}` should be readable: {error}"));
        assert!(
            struct_has_local_repr_c(&source, table_name),
            "`{table_name}` in `{relative_file}` should keep a local #[repr(C)]"
        );
        let field_count = public_struct_field_count(&source, table_name);
        assert_eq!(
            field_count, *expected_fields,
            "`{table_name}` in `{relative_file}` should keep its documented Runtime 10 field count"
        );
    }
}

fn assert_runtime_10_ffi_wrappers(
    exports_source: &str,
    session_source: &str,
    operation_source: &str,
) {
    assert_eq!(
        EXPECTED_RUNTIME_10_SESSION_OPERATIONS.len(),
        22,
        "Runtime 10 session operation inventory should stay at 22 operations"
    );
    for operation in EXPECTED_RUNTIME_10_SESSION_OPERATIONS {
        let wrapper = format!("{operation}_ffi");
        assert!(
            exports_source.contains(&format!("Some({wrapper})")),
            "`ZrRuntimeApiV6` should advertise `{wrapper}`"
        );
        assert!(
            exports_source.contains(&format!("fn {wrapper}(")),
            "`exports.rs` should keep wrapper function `{wrapper}`"
        );
        assert!(
            exports_source.contains(&format!("catch_ffi_panic(|| unsafe {{ {operation}(")),
            "`{wrapper}` should call `{operation}` inside catch_ffi_panic"
        );
        assert!(
            !exports_source.contains(&format!("Some({operation}),")),
            "`ZrRuntimeApiV6` must not advertise `{operation}` directly"
        );
        let owner_source = if operation.ends_with("_operation") {
            operation_source
        } else {
            session_source
        };
        let expected_visibility = if operation.ends_with("_operation") {
            "pub(crate)"
        } else {
            "pub(in crate::dynamic_api)"
        };
        assert!(
            owner_source.contains(&format!("{expected_visibility} unsafe fn {operation}(")),
            "the session FFI owner should keep private Rust ABI owner `{operation}`"
        );
    }
    assert!(
        !session_source.contains("pub(in crate::dynamic_api) unsafe extern \"C\" fn"),
        "private dynamic session owner functions must not become extern C"
    );
    assert!(
        !operation_source.contains("pub(crate) unsafe extern \"C\" fn"),
        "private operation owner functions must not become extern C"
    );
}

fn assert_runtime_10_v6_only_hard_cutover(repo_root: &Path) {
    const PRODUCTION_OWNERS: &[&str] = &[
        "zircon_runtime_interface/src/runtime_api/api_table.rs",
        "zircon_runtime/src/dynamic_api/exports.rs",
        "zircon_app/src/entry/runtime_library/loaded_runtime.rs",
    ];
    const LEGACY_SYMBOLS: &[&str] = &[
        "ZrRuntimeApiV1",
        "ZrRuntimeGetApiFnV1",
        "ZR_RUNTIME_GET_API_SYMBOL_V1",
        "zircon_runtime_get_api_v1",
        "RuntimeApi::V1",
        "ZrRuntimeApiV2",
        "ZrRuntimeGetApiFnV2",
        "ZR_RUNTIME_GET_API_SYMBOL_V2",
        "zircon_runtime_get_api_v2",
        "RuntimeApi::V2",
        "ZrRuntimeApiV3",
        "ZrRuntimeGetApiFnV3",
        "ZR_RUNTIME_GET_API_SYMBOL_V3",
        "zircon_runtime_get_api_v3",
        "RuntimeApi::V3",
        "ZrRuntimeApiV4",
        "ZrRuntimeGetApiFnV4",
        "ZR_RUNTIME_GET_API_SYMBOL_V4",
        "zircon_runtime_get_api_v4",
        "RuntimeApi::V4",
    ];

    for relative_file in PRODUCTION_OWNERS {
        let source = fs::read_to_string(repo_root.join(relative_file))
            .unwrap_or_else(|error| panic!("`{relative_file}` should be readable: {error}"));
        for legacy_symbol in LEGACY_SYMBOLS {
            assert!(
                !source.contains(legacy_symbol),
                "`{relative_file}` must not restore legacy runtime API symbol `{legacy_symbol}`"
            );
        }
    }
}

fn source_tree_contains(repo_root: &Path, needle: &str) -> bool {
    EXPECTED_RUNTIME_10_SOURCE_FILES
        .iter()
        .chain(EXTRA_RUNTIME_10_GUARD_ANCHOR_FILES.iter())
        .any(|relative_file| {
            fs::read_to_string(repo_root.join(relative_file))
                .map(|source| source.contains(needle))
                .unwrap_or(false)
        })
}

fn struct_has_local_repr_c(source: &str, struct_name: &str) -> bool {
    let struct_anchor = format!("pub struct {struct_name} {{");
    let Some(struct_start) = source.find(&struct_anchor) else {
        return false;
    };
    let prefix = &source[..struct_start];
    let Some(repr_index) = prefix.rfind("#[repr(C)]") else {
        return false;
    };
    match prefix.rfind("pub struct ") {
        Some(previous_struct_index) => previous_struct_index < repr_index,
        None => true,
    }
}

fn public_struct_field_count(source: &str, struct_name: &str) -> usize {
    let struct_anchor = format!("pub struct {struct_name} {{");
    let body_start = source
        .find(&struct_anchor)
        .unwrap_or_else(|| panic!("source should contain struct anchor `{struct_anchor}`"))
        + struct_anchor.len();
    let body_tail = &source[body_start..];
    let body_end = body_tail
        .find("\n}")
        .unwrap_or_else(|| panic!("source should contain closing brace for `{struct_name}`"));
    let body = &body_tail[..body_end];
    body.lines()
        .filter(|line| line.trim_start().starts_with("pub "))
        .count()
}
