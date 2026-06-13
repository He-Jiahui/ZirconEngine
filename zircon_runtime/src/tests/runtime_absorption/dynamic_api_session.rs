use std::fs;
use std::path::Path;

const EXPECTED_RUNTIME_10_SOURCE_FILES: &[&str] = &[
    "zircon_runtime_interface/src/runtime_api/api_table.rs",
    "zircon_runtime_interface/src/plugin_api.rs",
    "zircon_runtime_interface/src/tests/abi_safety_contracts.rs",
    "zircon_runtime/src/dynamic_api/exports.rs",
    "zircon_runtime/src/dynamic_api/session.rs",
    "zircon_runtime/src/dynamic_api/tests/api_table.rs",
    "zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs",
    "zircon_runtime/src/dynamic_api/tests/session_entry_points.rs",
    "zircon_runtime/src/dynamic_api/tests/session_profiles.rs",
    "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs",
    "zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late.rs",
    "zircon_app/src/entry/runtime_library/loaded_runtime.rs",
    "zircon_app/src/entry/runtime_library/runtime_session.rs",
    "zircon_app/src/entry/runtime_library/tests.rs",
];

const EXPECTED_RUNTIME_10_FUNCTION_TABLES: &[(&str, &str, usize)] = &[
    (
        "zircon_runtime_interface/src/runtime_api/api_table.rs",
        "ZrHostApiV1",
        4,
    ),
    (
        "zircon_runtime_interface/src/runtime_api/api_table.rs",
        "ZrRuntimeApiV1",
        13,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrHostApiV3",
        7,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrHostEcsApiV1",
        3,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrHostAssetApiV1",
        1,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrHostEventApiV1",
        2,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrHostBridgeApiV1",
        1,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrHostDiagnosticsApiV1",
        2,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrPluginStateSnapshotApiV1",
        4,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrPluginApiV1",
        4,
    ),
];

const EXPECTED_RUNTIME_10_SESSION_OPERATIONS: &[&str] = &[
    "create_session",
    "destroy_session",
    "handle_event",
    "capture_frame",
    "capture_accessibility_tree",
    "bind_viewport_surface",
    "unbind_viewport_surface",
    "present_viewport",
    "profile_control",
    "tick_frame",
    "drain_host_requests",
];

const EXPECTED_RUNTIME_10_MIRROR_DOCS: &[&str] = &[
    "docs/zircon_runtime/dynamic_api/session.md",
    "docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md",
    "docs/plans/zircon_runtime/runtime/index.md",
    "docs/engine-architecture/runtime-architecture-review-m0.md",
    "docs/engine-architecture/runtime-interface-convergence.md",
    "docs/engine-architecture/runtime-interface-cdylib-loader.md",
];

fn slice_between<'a>(source: &'a str, start_anchor: &str, end_anchor: &str) -> &'a str {
    let start = source
        .find(start_anchor)
        .unwrap_or_else(|| panic!("source should contain start anchor `{start_anchor}`"));
    let tail = &source[start..];
    let end = tail.find(end_anchor).unwrap_or_else(|| {
        panic!("source should contain end anchor `{end_anchor}` after `{start_anchor}`")
    });
    &tail[..end]
}

#[test]
fn runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces() {
    let session_source = include_str!("../../dynamic_api/session.rs");
    let lifecycle_tests = include_str!("../../dynamic_api/tests/session_lifecycle.rs");
    let session_profile_tests = include_str!("../../dynamic_api/tests/session_profiles.rs");
    let session_entry_point_tests = include_str!("../../dynamic_api/tests/session_entry_points.rs");
    let session_doc = include_str!("../../../../docs/zircon_runtime/dynamic_api/session.md");
    let runtime_10_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");

    for required_source_anchor in [
        "render_bridge: Option<RuntimeRenderBridge>",
        "RUNTIME_SESSION_PROFILE_MINIMAL",
        "RUNTIME_SESSION_PROFILE_HEADLESS",
        "fn uses_render_bridge(self) -> bool",
        "runtime_dynamic_session_render_bridge_skipped",
    ] {
        assert!(
            session_source.contains(required_source_anchor),
            "dynamic session source should keep Runtime 10 headless anchor `{required_source_anchor}`"
        );
    }

    let uses_render_bridge = slice_between(
        session_source,
        "fn uses_render_bridge(self) -> bool",
        "\n    }\n}",
    );
    assert!(
        uses_render_bridge.contains("matches!(self, Self::Runtime | Self::Editor | Self::Dev)"),
        "only rendered runtime/editor/dev profiles should create RuntimeRenderBridge"
    );
    for forbidden_profile in ["Self::Minimal", "Self::Headless"] {
        assert!(
            !uses_render_bridge.contains(forbidden_profile),
            "headless lifecycle profiles must not re-enter RuntimeRenderBridge creation through `{forbidden_profile}`"
        );
    }

    let construction = slice_between(
        session_source,
        "let render_bridge = if profile.uses_render_bridge()",
        "let level = {",
    );
    for required_construction_anchor in [
        "RuntimeRenderBridge::new(&core)",
        "Some(render_bridge)",
        "runtime_dynamic_session_render_bridge_skipped",
        "None",
    ] {
        assert!(
            construction.contains(required_construction_anchor),
            "dynamic session construction should keep optional render bridge anchor `{required_construction_anchor}`"
        );
    }

    let capture_frame = slice_between(
        session_source,
        "    fn capture_frame(",
        "    fn bind_viewport_surface",
    );
    for required_capture_anchor in [
        "if let Some(render_bridge) = &mut self.render_bridge",
        "submit_extract_with_ui",
        "empty_captured_frame(requested)",
    ] {
        assert!(
            capture_frame.contains(required_capture_anchor),
            "headless capture should keep empty-frame fallback anchor `{required_capture_anchor}`"
        );
    }

    for (method_start, method_end) in [
        (
            "    fn bind_viewport_surface(",
            "    fn unbind_viewport_surface(",
        ),
        (
            "    fn unbind_viewport_surface(",
            "    fn present_viewport(",
        ),
        (
            "    fn present_viewport(&mut self",
            "    fn capture_accessibility_tree(",
        ),
    ] {
        let method = slice_between(session_source, method_start, method_end);
        assert!(
            method.contains("let Some(render_bridge) = &mut self.render_bridge else"),
            "`{method_start}` should gate WGPU work on an installed RuntimeRenderBridge"
        );
        assert!(
            method.contains("return Ok(());"),
            "`{method_start}` should be a no-op when headless/minimal skipped the render bridge"
        );
    }

    for (test_source, required_test_anchor) in [
        (
            session_profile_tests,
            "create_session_accepts_named_headless_profile_without_render_bridge",
        ),
        (
            session_profile_tests,
            "minimal_and_headless_profiles_skip_render_bridge_bootstrap",
        ),
        (
            session_entry_point_tests,
            "destroyed_headless_session_entry_points_reject_old_handle",
        ),
        (
            lifecycle_tests,
            "session_destroy_reports_explicit_not_found_after_headless_destroy",
        ),
    ] {
        assert!(
            test_source.contains(required_test_anchor),
            "dynamic API lifecycle tests should keep Runtime 10 evidence `{required_test_anchor}`"
        );
    }

    for required_doc_anchor in [
        "minimal` and `headless` profiles now skip `RuntimeRenderBridge` creation",
        "frame capture returns an empty encoded frame",
        "surface bind/unbind/present operations are no-ops",
        "runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces",
    ] {
        assert!(
            session_doc.contains(required_doc_anchor),
            "dynamic API session docs should record `{required_doc_anchor}`"
        );
    }

    for required_plan_anchor in [
        "headless/minimal profile 明确跳过 render bridge",
        "capture 返回空帧",
        "surface bind/unbind/present 为 no-op",
        "runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces",
    ] {
        assert!(
            runtime_10_plan.contains(required_plan_anchor)
                || runtime_index.contains(required_plan_anchor),
            "Runtime 10 plan status should record `{required_plan_anchor}`"
        );
    }
}

#[test]
fn runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge() {
    let exports_source = include_str!("../../dynamic_api/exports.rs");
    let session_source = include_str!("../../dynamic_api/session.rs");
    let api_table_tests = include_str!("../../dynamic_api/tests/api_table.rs");
    let session_doc = include_str!("../../../../docs/zircon_runtime/dynamic_api/session.md");
    let runtime_10_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");

    for required_exports_anchor in [
        "fn catch_ffi_panic(",
        "catch_unwind(AssertUnwindSafe",
        "ZrStatusCode::Panic",
        "runtime dynamic API panic caught at FFI boundary",
        "zircon_runtime_get_api_v1_inner",
        "Err(_) => core::ptr::null()",
    ] {
        assert!(
            exports_source.contains(required_exports_anchor),
            "dynamic API exports should keep FFI panic boundary anchor `{required_exports_anchor}`"
        );
    }

    for (inner, wrapper) in [
        ("create_session", "create_session_ffi"),
        ("destroy_session", "destroy_session_ffi"),
        ("handle_event", "handle_event_ffi"),
        ("capture_frame", "capture_frame_ffi"),
        (
            "capture_accessibility_tree",
            "capture_accessibility_tree_ffi",
        ),
        ("bind_viewport_surface", "bind_viewport_surface_ffi"),
        ("unbind_viewport_surface", "unbind_viewport_surface_ffi"),
        ("present_viewport", "present_viewport_ffi"),
        ("profile_control", "profile_control_ffi"),
        ("tick_frame", "tick_frame_ffi"),
        ("drain_host_requests", "drain_host_requests_ffi"),
    ] {
        assert!(
            exports_source.contains(&format!("Some({wrapper})")),
            "`ZrRuntimeApiV1` should advertise `{wrapper}` instead of the session owner `{inner}`"
        );
        assert!(
            exports_source.contains(&format!("fn {wrapper}(")),
            "dynamic API exports should keep wrapper function `{wrapper}`"
        );
        assert!(
            exports_source.contains(&format!("catch_ffi_panic(|| unsafe {{ {inner}(")),
            "`{wrapper}` should delegate to `{inner}` inside catch_ffi_panic"
        );
        assert!(
            !exports_source.contains(&format!("Some({inner}),")),
            "`ZrRuntimeApiV1` must not bypass the panic wrapper by advertising `{inner}` directly"
        );
        assert!(
            session_source.contains(&format!("pub(super) unsafe fn {inner}(")),
            "private dynamic session owner `{inner}` should stay Rust ABI so the exports wrapper can catch unwinds"
        );
    }

    assert!(
        !session_source.contains("pub(super) unsafe extern \"C\" fn"),
        "private dynamic session owner functions must not drift back to extern C"
    );

    for required_test_anchor in [
        "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
        "pub(super) unsafe extern \\\"C\\\" fn",
        "pub(super) unsafe fn {inner}(",
    ] {
        assert!(
            api_table_tests.contains(required_test_anchor),
            "dynamic API table tests should keep FFI panic guard evidence `{required_test_anchor}`"
        );
    }

    for required_doc_anchor in [
        "## FFI Panic Boundary",
        "Rust-ABI session owner functions",
        "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
        "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
    ] {
        assert!(
            session_doc.contains(required_doc_anchor),
            "dynamic API session docs should record `{required_doc_anchor}`"
        );
    }

    for required_plan_anchor in [
        "M1.3 FFI panic 边界",
        "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
        "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
        "session.rs` owner 函数保持 Rust ABI",
    ] {
        assert!(
            runtime_10_plan.contains(required_plan_anchor)
                || runtime_index.contains(required_plan_anchor),
            "Runtime 10 plan/index should record `{required_plan_anchor}`"
        );
    }
}

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
        fs::read_to_string(repo_root.join("zircon_runtime/src/dynamic_api/session.rs"))
            .expect("dynamic API session source should be readable");

    assert_runtime_10_files_exist(repo_root, EXPECTED_RUNTIME_10_SOURCE_FILES);
    assert_function_table_shapes(repo_root);
    assert_runtime_10_ffi_wrappers(&exports_source, &session_source);

    for required_anchor in [
        "runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces",
        "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
        "runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts",
        "runtime_10_m1_3_cargo_pending_gate_stays_explicit_until_dynamic_api_validation",
        "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
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
            "expected_source_file_count = 14",
            "function_table_structs = 10/10",
            "field_count_mismatches = 0",
            "missing_repr_c_tables = 0",
            "runtime_session_ffi_wrappers = 11/11",
            "direct_session_table_entry_bypasses = 0",
            "session_owner_extern_c_present = false",
            "headless_lifecycle_anchors = 12/12",
            "ffi_panic_anchors = 9/9",
            "loader_failure_anchors = 10/10",
            "ui_pending_gate_anchors = 8/8",
            "pending_cargo_gate_anchors = 5/5",
            "doc_anchors = 7/7",
            "mirror_docs_guard_present = true",
            "risks = []",
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
        14,
        "Runtime 10 dynamic API source inventory should stay at 14 files"
    );
    for relative_file in files {
        assert!(
            repo_root.join(relative_file).exists(),
            "Runtime 10 dynamic API source file `{relative_file}` should exist"
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

fn assert_runtime_10_ffi_wrappers(exports_source: &str, session_source: &str) {
    assert_eq!(
        EXPECTED_RUNTIME_10_SESSION_OPERATIONS.len(),
        11,
        "Runtime 10 session operation inventory should stay at 11 operations"
    );
    for operation in EXPECTED_RUNTIME_10_SESSION_OPERATIONS {
        let wrapper = format!("{operation}_ffi");
        assert!(
            exports_source.contains(&format!("Some({wrapper})")),
            "`ZrRuntimeApiV1` should advertise `{wrapper}`"
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
            "`ZrRuntimeApiV1` must not advertise `{operation}` directly"
        );
        assert!(
            session_source.contains(&format!("pub(super) unsafe fn {operation}(")),
            "`session.rs` should keep private Rust ABI owner `{operation}`"
        );
    }
    assert!(
        !session_source.contains("pub(super) unsafe extern \"C\" fn"),
        "private dynamic session owner functions must not become extern C"
    );
}

fn source_tree_contains(repo_root: &Path, needle: &str) -> bool {
    EXPECTED_RUNTIME_10_SOURCE_FILES
        .iter()
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
