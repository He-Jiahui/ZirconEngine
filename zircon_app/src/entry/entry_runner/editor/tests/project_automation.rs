use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(windows)]
use std::path::Path;

use zircon_runtime::asset::project::ProjectPaths;

use super::{
    normalize_resolved_project_automation_root, project_automation_report_path,
    require_healthy_project_for_automation, resolve_project_automation_input_path,
    EditorProjectAutomationReport, EditorProjectAutomationRequest, EditorProjectAutomationSnapshot,
};

#[test]
fn project_automation_fixture_roots_follow_the_resolved_test_binary_directory() {
    let root = automation_test_root("physical-root");
    let executable =
        std::env::current_exe().expect("locate the project-automation test executable");
    let binary_directory = executable
        .parent()
        .expect("project-automation test executable must have a parent directory");
    let resolved_binary_directory =
        zircon_runtime::asset::project::ProjectPaths::resolve_existing(binary_directory)
            .expect("resolve project-automation test binary directory");

    assert!(
        root.starts_with(resolved_binary_directory.operation_path()),
        "project-automation fixture output must retain the test binary's physical output root"
    );
}

fn automation_test_root(label: impl AsRef<str>) -> PathBuf {
    let executable =
        std::env::current_exe().expect("locate the project-automation test executable");
    let binary_directory = executable
        .parent()
        .expect("project-automation test executable must have a parent directory");
    let binary_directory =
        zircon_runtime::asset::project::ProjectPaths::resolve_existing(binary_directory)
            .expect("resolve the project-automation test binary directory");

    binary_directory
        .operation_path()
        .join("zircon-mvp-fixtures")
        .join(label.as_ref())
}

#[test]
fn project_automation_transfers_bindings_to_the_retained_host_before_report_serialization() {
    let source = include_str!("../project_automation.rs");
    let mut offset = 0;
    for needle in [
        "EditorApplicationComposition::open_resolved_project(project_root.clone())",
        ".map_err(|error| {",
        "project_root.display_diagnostic(error)",
        "composition.run_retained_host_automation(&request.bindings)",
        "let report = EditorProjectAuthoringAutomationReport",
        "Ok(report)",
    ] {
        let index = source[offset..]
            .find(needle)
            .unwrap_or_else(|| panic!("project automation lifecycle is missing `{needle}`"));
        offset += index + needle.len();
    }
    assert!(
        !source.contains(".dispatch_binding("),
        "app automation must not bypass retained-host callbacks"
    );
    assert!(
        !source.contains("open_project(project_root.operation_path())"),
        "automation must carry its resolved project identity into composition"
    );
}

#[test]
fn project_automation_manifest_input_derives_its_resolved_parent_without_reresolving() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    let project_root =
        automation_test_root(format!("manifest-input-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&project_root).expect("temporary project root should be created");
    let manifest = project_root.join(zircon_runtime::asset::project::PROJECT_MANIFEST_FILE);
    fs::write(&manifest, "name = 'automation-fixture'\n")
        .expect("temporary project manifest should be written");

    let resolved_manifest = resolve_project_automation_input_path(manifest, "project root")
        .expect("existing manifest input should resolve");
    let project_root = normalize_resolved_project_automation_root(resolved_manifest.clone())
        .expect("resolved manifest input should derive its project directory");

    assert_eq!(
        project_root.operation_path(),
        resolved_manifest
            .operation_path()
            .parent()
            .expect("manifest should have a parent directory")
    );
    assert_eq!(
        project_root.display_path(),
        resolved_manifest
            .display_path()
            .parent()
            .expect("display manifest should have a parent directory")
    );

    fs::remove_dir_all(project_root.operation_path())
        .expect("temporary project root should be removed");
}

#[test]
fn project_automation_keeps_a_manifest_named_directory_as_the_project_root() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    let location = automation_test_root(format!(
        "manifest-named-root-{}-{nonce}",
        std::process::id()
    ));
    let project_root = location.join(zircon_runtime::asset::project::PROJECT_MANIFEST_FILE);
    fs::create_dir_all(&project_root).expect("temporary project root should be created");

    let resolved = resolve_project_automation_input_path(project_root.clone(), "project root")
        .expect("existing project root should resolve");
    let normalized = normalize_resolved_project_automation_root(resolved)
        .expect("directory input should remain the project root");

    assert_eq!(
        normalized,
        ProjectPaths::resolve_existing(&project_root).unwrap()
    );
    fs::remove_dir_all(location).expect("temporary project root should be removed");
}

#[test]
fn project_automation_entry_delegates_the_typed_commandlet_to_the_process_host() {
    let source = include_str!("../../editor.rs");
    let mut offset = 0;
    for needle in [
        "EditorLaunchRoute::Commandlet(request) => {",
        "let commandlet_host = project_automation::EditorProjectAutomationCommandletHost;",
        "run_commandlet_with_host(request, &commandlet_host)",
        "println!(\"{}\", serde_json::to_string(&report)?);",
        "return Ok(report.exit_code().as_u8());",
    ] {
        let index = source[offset..]
            .find(needle)
            .unwrap_or_else(|| panic!("project automation commandlet entry is missing `{needle}`"));
        offset += index + needle.len();
    }
}

#[test]
fn project_automation_commandlet_host_resolves_typed_paths_before_opening_composition() {
    let source = include_str!("../project_automation.rs");
    let mut offset = 0;
    for needle in [
        "impl CommandletHost for EditorProjectAutomationCommandletHost",
        "request.project_root().to_path_buf()",
        "request.automation_path().to_path_buf()",
        "EditorApplicationComposition::open_resolved_project(project_root.clone())",
        "composition.run_retained_host_automation(&request.bindings)",
    ] {
        let index = source[offset..]
            .find(needle)
            .unwrap_or_else(|| panic!("project automation commandlet host is missing `{needle}`"));
        offset += index + needle.len();
    }
}

#[test]
fn empty_project_automation_request_is_rejected_before_project_open() {
    let request = EditorProjectAutomationRequest {
        bindings: vec![],
        product_workbench_capture: None,
    };

    assert_eq!(
        request.validate().unwrap_err().to_string(),
        "project-scoped editor automation requires at least one UI binding"
    );
}

#[test]
fn project_automation_report_path_uses_dot_for_the_current_project_root() {
    let current_directory = std::env::current_dir().expect("test process must have a cwd");

    assert_eq!(project_automation_report_path(&current_directory), ".");
}

#[cfg(windows)]
#[test]
fn project_automation_report_path_hides_windows_verbatim_prefixes() {
    assert_eq!(
        project_automation_report_path(Path::new(r"\\?\C:\ZirconBuilds\stage\project")),
        r"C:\ZirconBuilds\stage\project"
    );
    assert_eq!(
        project_automation_report_path(Path::new(r"\\?\UNC\server\share\project")),
        r"\\server\share\project"
    );
}

#[cfg(windows)]
#[test]
fn project_automation_open_errors_use_the_resolved_project_display_view() {
    let project = ProjectPaths::resolve_path(r"\\?\C:\ZirconBuilds\stage\project")
        .expect("Windows project path should resolve");

    assert_eq!(
        project.display_diagnostic(
            r"project manifest is missing: \\?\C:\ZirconBuilds\stage\project\zircon-project.toml"
        ),
        r"project manifest is missing: C:\ZirconBuilds\stage\project\zircon-project.toml"
    );
}

#[cfg(windows)]
#[test]
fn project_automation_input_resolution_rejects_drive_relative_paths() {
    let error = resolve_project_automation_input_path(PathBuf::from(r"C:automation.json"), "file")
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "could not resolve project automation file 'C:automation.json': Windows project paths must be drive-rooted, not drive-relative: C:automation.json"
    );
}

#[test]
fn project_automation_rejects_degraded_project_opens_before_binding_dispatch() {
    require_healthy_project_for_automation("Project opened: Fixture", 4, 4, 0).unwrap();

    for (status, assets, ready, failed) in [
        ("Project opened (degraded): Fixture", 4, 4, 0),
        ("Project opened: Fixture", 4, 3, 0),
        ("Project opened: Fixture", 4, 4, 1),
    ] {
        let error = require_healthy_project_for_automation(status, assets, ready, failed)
            .expect_err("degraded project must not accept automation bindings");

        assert!(error.to_string().contains("non-degraded project open"));
    }
}

#[test]
fn project_automation_report_serializes_the_compact_editor_snapshot() {
    let report =
        EditorProjectAutomationReport::Authoring(super::EditorProjectAuthoringAutomationReport {
            project_path: "fixture-project".to_string(),
            project_identity: "Fixture".to_string(),
            manifest_identity: "Fixture@v1".to_string(),
            scene_uri: "res://scenes/main.scene.toml".to_string(),
            selected_model_resource_id: Some("model-id".to_string()),
            selected_material_resource_id: Some("material-id".to_string()),
            opened_project_inspection_generation: Some(1),
            records: vec![],
            snapshot: EditorProjectAutomationSnapshot {
                project_open: true,
                scene_entry_count: 3,
                selected_node_id: Some(3),
                selected_node_name: Some("Cube".to_string()),
                inspector_translation: Some(["42".to_string(), "0".to_string(), "0".to_string()]),
                inspector_scale: Some(["1.25".to_string(), "1".to_string(), "1".to_string()]),
                scene_nodes: vec![],
            },
        });

    let value = serde_json::to_value(report).expect("report serialization should succeed");
    assert_eq!(value["snapshot"]["selected_node_id"], 3);
    assert_eq!(value["snapshot"]["selected_node_name"], "Cube");
    assert_eq!(value["snapshot"]["inspector_translation"][0], "42");
    assert_eq!(value["snapshot"]["inspector_scale"][0], "1.25");
    assert_eq!(value["snapshot"]["scene_nodes"], serde_json::json!([]));
    assert_eq!(value["project_identity"], "Fixture");
    assert_eq!(value["manifest_identity"], "Fixture@v1");
    assert_eq!(value["scene_uri"], "res://scenes/main.scene.toml");
    assert_eq!(value["selected_model_resource_id"], "model-id");
    assert_eq!(value["selected_material_resource_id"], "material-id");
}

#[test]
fn project_automation_source_bound_f5_requests_deserialize_to_normal_binding_sequences() {
    let authoring: EditorProjectAutomationRequest = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../tools/mvp/mvp-authoring-automation.json"
    )))
    .expect("the source-bound F5 authoring request must use the editor binding schema");
    let reopen: EditorProjectAutomationRequest = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../tools/mvp/mvp-reopen-automation.json"
    )))
    .expect("the source-bound F5 reopen request must use the editor binding schema");

    assert_eq!(
        authoring
            .bindings
            .iter()
            .map(|binding| binding.native_binding())
            .collect::<Vec<_>>(),
        vec![
            "Hierarchy/SelectCube:onClick",
            "Inspector/TransformPositionXCommit:onSubmit",
            "Inspector/TransformScaleXCommit:onSubmit",
            "WorkbenchMenuBar/Undo:onClick",
            "WorkbenchMenuBar/Redo:onClick",
            "WorkbenchMenuBar/SaveProject:onClick",
        ]
    );
    assert_eq!(
        reopen
            .bindings
            .iter()
            .map(|binding| binding.native_binding())
            .collect::<Vec<_>>(),
        vec!["Hierarchy/SelectCube:onClick"]
    );
}

#[test]
fn product_capture_request_is_additive_and_rejects_empty_or_mixed_operations() {
    let repo_root = std::env::current_dir().expect("test repo root should be absolute");
    let request: EditorProjectAutomationRequest = serde_json::from_value(serde_json::json!({
        "bindings": [],
        "productWorkbenchCapture": {
            "repoRoot": repo_root,
            "snapshotsOutputPath": "workbench-product-state.json",
            "captureZuiVisualEvidence": true
        }
    }))
    .expect("product capture request should deserialize");
    request
        .validate()
        .expect("snapshot and native capture can run together");

    let no_operation: EditorProjectAutomationRequest = serde_json::from_value(serde_json::json!({
        "bindings": [],
        "productWorkbenchCapture": { "repoRoot": std::env::current_dir().unwrap() }
    }))
    .expect("capture request with no operation should deserialize before validation");
    assert!(no_operation.validate().is_err());

    let mixed: EditorProjectAutomationRequest = serde_json::from_value(serde_json::json!({
        "bindings": [{
            "path": {
                "view_id": "Hierarchy",
                "control_id": "SelectCube",
                "event_kind": "Click"
            },
            "payload": {
                "SelectionCommand": {
                    "SelectSceneNode": {"node_id": 3}
                }
            }
        }],
        "productWorkbenchCapture": {
            "repoRoot": std::env::current_dir().unwrap(),
            "captureZuiVisualEvidence": true
        }
    }))
    .expect("mixed capture and binding request should deserialize before validation");
    assert!(mixed.validate().is_err());
}
