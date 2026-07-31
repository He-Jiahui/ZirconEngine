use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const FUNCTION_TABLE_SOURCES: &[(&str, &[&str])] = &[
    (
        "src/runtime_api/api_table.rs",
        &["ZrHostApiV1", "ZrRuntimeApiV3"],
    ),
    (
        "src/plugin_api.rs",
        &[
            "ZrHostApiV3",
            "ZrHostApiV4",
            "ZrHostEcsApiV1",
            "ZrHostEcsApiV2",
            "ZrHostAssetApiV1",
            "ZrHostEventApiV1",
            "ZrHostBridgeApiV1",
            "ZrHostDiagnosticsApiV1",
            "ZrPluginStateSnapshotApiV1",
            "ZrPluginApiV1",
        ],
    ),
];
const FUNCTION_TABLE_FIELD_COUNTS: &[(&str, &str, usize)] = &[
    ("src/runtime_api/api_table.rs", "ZrHostApiV1", 4),
    ("src/runtime_api/api_table.rs", "ZrRuntimeApiV3", 19),
    ("src/plugin_api.rs", "ZrHostApiV3", 7),
    ("src/plugin_api.rs", "ZrHostApiV4", 7),
    ("src/plugin_api.rs", "ZrHostEcsApiV1", 3),
    ("src/plugin_api.rs", "ZrHostEcsApiV2", 3),
    ("src/plugin_api.rs", "ZrHostAssetApiV1", 1),
    ("src/plugin_api.rs", "ZrHostEventApiV1", 2),
    ("src/plugin_api.rs", "ZrHostBridgeApiV1", 1),
    ("src/plugin_api.rs", "ZrHostDiagnosticsApiV1", 2),
    ("src/plugin_api.rs", "ZrPluginStateSnapshotApiV1", 4),
    ("src/plugin_api.rs", "ZrPluginApiV1", 4),
];
const RUNTIME_API_V3_SESSION_OPERATION_FIELDS: &[&str] = &[
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
    "subscribe_plugin_event",
    "unsubscribe_plugin_event",
    "drain_plugin_events",
    "submit_operation",
    "poll_operation",
    "harvest_operation",
];
const FORBIDDEN_PUBLIC_SIGNATURE_NEEDLES: &[&str] = &["Box<dyn", "Rc<", "Arc<dyn", "impl Trait"];

#[test]
fn function_table_structs_are_all_repr_c() {
    let expected = expected_function_table_names();
    let mut discovered = BTreeSet::new();

    for &(source_path, expected_names) in FUNCTION_TABLE_SOURCES {
        let source = read_manifest_source(source_path);
        for &name in expected_names {
            assert_repr_c_before_struct(&source, source_path, name);
            discovered.insert(name.to_string());
        }
        for name in discover_api_struct_names(&source) {
            discovered.insert(name);
        }
    }

    assert_eq!(
        discovered, expected,
        "runtime/interface function-table inventory changed; update the runtime 10 ABI matrix before changing the guard"
    );
}

#[test]
fn interface_public_signatures_stay_free_of_dynamic_object_exports() {
    let mut violations = Vec::new();

    for source_path in production_rust_sources() {
        let source = std::fs::read_to_string(&source_path).expect("read interface source");
        violations.extend(forbidden_public_signature_violations(&source_path, &source));
    }

    assert!(
        violations.is_empty(),
        "zircon_runtime_interface public signatures must stay ABI-safe:\n{}",
        violations.join("\n")
    );
}

#[test]
fn function_table_field_counts_match_runtime_10_inventory() {
    for &(source_path, table_name, expected_field_count) in FUNCTION_TABLE_FIELD_COUNTS {
        let source = read_manifest_source(source_path);
        let fields = discover_struct_fields(&source, source_path, table_name);

        assert_eq!(
            fields.len(),
            expected_field_count,
            "{source_path}::{table_name} field count changed; update the runtime 10 ABI inventory and version strategy before changing the guard"
        );
    }
}

#[test]
fn runtime_api_session_operation_surface_matches_inventory() {
    let source = read_manifest_source("src/runtime_api/api_table.rs");
    let v3_fields =
        discover_struct_fields(&source, "src/runtime_api/api_table.rs", "ZrRuntimeApiV3");
    let v3_operation_fields = v3_fields
        .iter()
        .filter_map(|field| match field.as_str() {
            "abi_version" | "size_bytes" => None,
            name => Some(name),
        })
        .collect::<Vec<_>>();

    assert_eq!(
        v3_operation_fields, RUNTIME_API_V3_SESSION_OPERATION_FIELDS,
        "ZrRuntimeApiV3 session operation surface changed; update runtime 10 docs and failure-path tests before changing the guard"
    );
}

#[test]
fn runtime_v3_reactive_wake_dtos_keep_explicit_c_layout_and_raw_kind() {
    let session_source = read_manifest_source("src/runtime_api/session.rs");
    let demand_source = read_manifest_source("src/runtime_api/frame_demand.rs");

    assert_repr_c_before_struct(
        &session_source,
        "src/runtime_api/session.rs",
        "ZrRuntimeWakeSinkV1",
    );
    assert_repr_c_before_struct(
        &session_source,
        "src/runtime_api/session.rs",
        "ZrRuntimeSessionConfigV2",
    );
    assert_repr_c_before_struct(
        &demand_source,
        "src/runtime_api/frame_demand.rs",
        "ZrRuntimeFrameDemandV1",
    );
    assert_eq!(
        discover_struct_fields(
            &session_source,
            "src/runtime_api/session.rs",
            "ZrRuntimeWakeSinkV1",
        ),
        ["abi_version", "token", "wake"]
            .map(str::to_string)
            .to_vec()
    );
    assert_eq!(
        discover_struct_fields(
            &session_source,
            "src/runtime_api/session.rs",
            "ZrRuntimeSessionConfigV2",
        ),
        ["abi_version", "profile", "project_manifest", "wake_sink"]
            .map(str::to_string)
            .to_vec()
    );
    assert_eq!(
        discover_struct_fields(
            &demand_source,
            "src/runtime_api/frame_demand.rs",
            "ZrRuntimeFrameDemandV1",
        ),
        ["abi_version", "kind", "delay_nanoseconds"]
            .map(str::to_string)
            .to_vec()
    );
    assert!(session_source.contains("pub wake: Option<unsafe extern \"C\" fn(u64)>,"));
    assert!(demand_source.contains("pub kind: u32,"));
    assert!(!demand_source.contains("pub enum ZrRuntimeFrameDemand"));
}

#[test]
fn runtime_table_v2_export_and_loader_fallback_stay_hard_deleted() {
    let sources = [
        (
            "zircon_runtime_interface/src/runtime_api/api_table.rs",
            read_manifest_source("src/runtime_api/api_table.rs"),
        ),
        (
            "zircon_runtime_interface/src/lib.rs",
            read_manifest_source("src/lib.rs"),
        ),
        (
            "zircon_runtime_interface/src/version.rs",
            read_manifest_source("src/version.rs"),
        ),
        (
            "zircon_runtime/src/dynamic_api/exports.rs",
            read_repo_file("zircon_runtime/src/dynamic_api/exports.rs"),
        ),
        (
            "zircon_runtime/src/dynamic_api/mod.rs",
            read_repo_file("zircon_runtime/src/dynamic_api/mod.rs"),
        ),
        (
            "zircon_runtime/src/dynamic_api/session/ffi.rs",
            read_repo_file("zircon_runtime/src/dynamic_api/session/ffi.rs"),
        ),
        (
            "zircon_app/src/entry/runtime_library/loaded_runtime.rs",
            read_repo_file("zircon_app/src/entry/runtime_library/loaded_runtime.rs"),
        ),
        (
            "zircon_app/src/entry/runtime_library/runtime_session.rs",
            read_repo_file("zircon_app/src/entry/runtime_library/runtime_session.rs"),
        ),
    ];
    let forbidden = [
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
        "ZrRuntimeSessionConfigV1",
        "ZrRuntimeCreateSessionFnV1",
        "ZrRuntimeTickFrameFnV1",
    ];
    let violations = sources
        .iter()
        .flat_map(|(path, source)| {
            forbidden.iter().filter_map(move |needle| {
                source.contains(needle).then(|| format!("{path}: {needle}"))
            })
        })
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "the runtime table is V3-only; remove the V2 table/export/loader fallback and retired signatures:\n{}",
        violations.join("\n")
    );
}

#[test]
fn runtime_10_version_strategy_rejects_in_place_table_shape_changes() {
    let convergence_doc =
        read_repo_file("docs/engine-architecture/runtime-interface-convergence.md");
    let version_source = read_manifest_source("src/version.rs");

    for phrase in [
        "field addition, removal, reorder, type change, or meaning change creates a new table version",
        "does not rely on silent tail-field extension under the same version",
        "`size_bytes` remains a diagnostic/validation field",
        "`ZIRCON_RUNTIME_ABI_VERSION_V1` continues to govern unchanged DTO records and `ZrHostApiV1`",
    ] {
        assert!(
            convergence_doc.contains(phrase),
            "runtime interface convergence doc should preserve the conservative ABI version rule phrase: {phrase}"
        );
    }
    assert!(
        version_source.contains("pub const ZIRCON_RUNTIME_ABI_VERSION_V1: u32 = 1;"),
        "runtime interface ABI version constant should remain the documented V1 owner"
    );
    assert!(
        version_source.contains("pub const ZIRCON_RUNTIME_ABI_VERSION_V2: u32 = 2;")
            && version_source.contains("pub const ZIRCON_RUNTIME_API_VERSION_V3: u32 = 3;"),
        "the V3 table and changed V2 session config need explicit version owners"
    );
    assert!(
        !version_source.contains("ZIRCON_RUNTIME_API_VERSION_V2"),
        "the retired V2 table version constant must not remain as a compatibility surface"
    );
}

#[test]
fn repr_c_guard_fails_on_missing_local_attribute() {
    let source = r#"
#[derive(Clone, Copy, Debug)]
pub struct ZrSyntheticApiV1 {
    pub abi_version: u32,
}
"#;

    assert!(
        std::panic::catch_unwind(|| {
            assert_repr_c_before_struct(source, "synthetic_api.rs", "ZrSyntheticApiV1");
        })
        .is_err(),
        "function table guard must reject a table without a local #[repr(C)]"
    );
}

#[test]
fn public_signature_guard_fails_on_dynamic_object_export() {
    let source = r#"
pub fn leak_callback(callback: Box<dyn FnOnce()>) {}
fn private_callback(callback: Box<dyn FnOnce()>) {}
"#;

    let violations = forbidden_public_signature_violations(Path::new("synthetic.rs"), source);

    assert_eq!(violations.len(), 1);
    assert!(violations[0].contains("Box<dyn"));
}

fn assert_repr_c_before_struct(source: &str, source_path: &str, name: &str) {
    let struct_needle = format!("pub struct {name} ");
    let struct_index = source.find(&struct_needle).unwrap_or_else(|| {
        panic!("{source_path} must keep function-table struct `{name}` in the ABI inventory")
    });
    let prefix = &source[..struct_index];
    let repr_index = prefix
        .rfind("#[repr(C)]")
        .unwrap_or_else(|| panic!("{source_path}::{name} must be declared with #[repr(C)]"));
    let previous_struct_index = prefix.rfind("pub struct ");

    assert!(
        previous_struct_index.is_none_or(|index| index < repr_index),
        "{source_path}::{name} must have a #[repr(C)] attribute in its own attribute block"
    );
}

fn discover_api_struct_names(source: &str) -> BTreeSet<String> {
    source
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let rest = line.strip_prefix("pub struct ")?;
            let name = rest.split_whitespace().next()?.trim_end_matches('{');
            if name.ends_with("ApiV1")
                || name.ends_with("ApiV2")
                || name.ends_with("ApiV3")
                || name.ends_with("ApiV4")
            {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn discover_struct_fields(source: &str, source_path: &str, name: &str) -> Vec<String> {
    let struct_needle = format!("pub struct {name} ");
    let struct_index = source.find(&struct_needle).unwrap_or_else(|| {
        panic!("{source_path} must keep function-table struct `{name}` in the ABI inventory")
    });
    let body_start = source[struct_index..]
        .find('{')
        .map(|offset| struct_index + offset + 1)
        .unwrap_or_else(|| panic!("{source_path}::{name} must have a struct body"));
    let body = &source[body_start..];
    let mut fields = Vec::new();

    for line in body.lines() {
        let line = line.trim();
        if line == "}" {
            break;
        }
        let Some(field) = line.strip_prefix("pub ") else {
            continue;
        };
        let field = field
            .split_once(':')
            .map(|(field, _)| field.trim())
            .filter(|field| !field.is_empty())
            .unwrap_or_else(|| panic!("{source_path}::{name} has an unparsable field line"));
        fields.push(field.to_string());
    }

    fields
}

fn forbidden_public_signature_violations(source_path: &Path, source: &str) -> Vec<String> {
    let mut violations = Vec::new();

    for (line_index, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("pub ") {
            continue;
        }
        for needle in FORBIDDEN_PUBLIC_SIGNATURE_NEEDLES {
            if trimmed.contains(needle) {
                violations.push(format!(
                    "{}:{} exports forbidden ABI boundary token `{needle}`",
                    relative_to_manifest(source_path).display(),
                    line_index + 1
                ));
            }
        }
    }

    violations
}

fn expected_function_table_names() -> BTreeSet<String> {
    FUNCTION_TABLE_SOURCES
        .iter()
        .flat_map(|(_, names)| names.iter().map(|name| (*name).to_string()))
        .collect()
}

fn read_manifest_source(path: &str) -> String {
    std::fs::read_to_string(manifest_dir().join(path)).expect("read ABI source")
}

fn read_repo_file(path: &str) -> String {
    std::fs::read_to_string(repo_root().join(path)).unwrap_or_else(|error| {
        panic!("expected to read {path}: {error}");
    })
}

fn production_rust_sources() -> Vec<PathBuf> {
    let source_root = manifest_dir().join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);
    sources
}

fn collect_rust_sources(path: &Path, sources: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(path).expect("read interface source directory") {
        let entry = entry.expect("read interface source entry");
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name == OsStr::new("tests"))
            {
                continue;
            }
            collect_rust_sources(&path, sources);
        } else if path
            .extension()
            .is_some_and(|extension| extension == OsStr::new("rs"))
        {
            sources.push(path);
        }
    }
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_root() -> PathBuf {
    manifest_dir()
        .parent()
        .expect("interface crate should have a workspace parent")
        .to_path_buf()
}

fn relative_to_manifest(path: &Path) -> PathBuf {
    path.strip_prefix(manifest_dir())
        .unwrap_or(path)
        .to_path_buf()
}
