use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use zircon_editor::{
    core::{
        commandlet::{AuthoringAutomationCommandletRequest, CommandletHost},
        editor_event::EditorEventRecord,
    },
    ui::binding::EditorUiBinding,
};
use zircon_runtime::asset::project::{ProjectPaths, ResolvedProjectPath};

use super::EditorApplicationComposition;

/// A project-scoped sequence of normal editor UI bindings.
///
/// The request intentionally carries the existing binding protocol rather than a separate
/// command language, so automation preserves the same dispatch, transaction, and save paths as
/// the retained editor host.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct EditorProjectAutomationRequest {
    pub bindings: Vec<EditorUiBinding>,
}

impl EditorProjectAutomationRequest {
    fn validate(&self) -> Result<(), io::Error> {
        if self.bindings.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "project-scoped editor automation requires at least one UI binding",
            ));
        }

        Ok(())
    }
}

/// Process host for the commandlet that drives normal retained-host bindings.
pub(crate) struct EditorProjectAutomationCommandletHost;

impl CommandletHost for EditorProjectAutomationCommandletHost {
    type AuthoringAutomationReport = EditorProjectAutomationReport;
    type Error = Box<dyn Error>;

    fn run_authoring_automation(
        &self,
        request: &AuthoringAutomationCommandletRequest,
    ) -> Result<Self::AuthoringAutomationReport, Self::Error> {
        let project_root =
            normalize_resolved_project_automation_root(resolve_project_automation_input_path(
                request.project_root().to_path_buf(),
                "project root",
            )?)?;
        let automation_path =
            resolve_project_automation_input_path(request.automation_path().to_path_buf(), "file")?;
        let contents = fs::read_to_string(automation_path.operation_path()).map_err(|error| {
            io::Error::other(format!(
                "could not read project automation file '{}': {error}",
                automation_path.display_path().display()
            ))
        })?;
        let request: EditorProjectAutomationRequest =
            serde_json::from_str(&contents).map_err(|error| {
                io::Error::other(format!(
                    "could not parse project automation file '{}': {error}",
                    automation_path.display_path().display()
                ))
            })?;
        request.validate()?;

        execute_project_automation(&project_root, &request)
    }
}

fn resolve_project_automation_input_path(
    path: PathBuf,
    label: &str,
) -> Result<ResolvedProjectPath, io::Error> {
    let display_path = ProjectPaths::display_path(&path);
    ProjectPaths::resolve_existing(&path).map_err(|error| {
        io::Error::other(format!(
            "could not resolve project automation {label} '{}': {error}",
            display_path.display()
        ))
    })
}

/// Accepts the same directory-or-manifest project input shape as `ProjectAuthority` while
/// retaining the resolved operation/display pair selected by the CLI boundary.
fn normalize_resolved_project_automation_root(
    path: ResolvedProjectPath,
) -> Result<ResolvedProjectPath, io::Error> {
    if ProjectPaths::is_project_manifest_file(path.operation_path()) {
        return path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "project manifest '{}' has no parent project directory",
                    path.display_path().display()
                ),
            )
        });
    }
    Ok(path)
}

/// Structured evidence from applying a normal binding sequence to one opened project.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct EditorProjectAutomationReport {
    pub project_path: String,
    pub project_identity: String,
    pub manifest_identity: String,
    pub scene_uri: String,
    pub selected_model_resource_id: Option<String>,
    pub selected_material_resource_id: Option<String>,
    pub opened_project_inspection_generation: Option<u64>,
    pub records: Vec<EditorEventRecord>,
    /// Retained-host and authoritative scene projection captured after final binding dispatch.
    /// A later process can compare the full persisted scene without bypassing project-open and
    /// selection paths.
    pub snapshot: EditorProjectAutomationSnapshot,
}

/// Stable editor and scene projection needed by product automation evidence.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct EditorProjectAutomationSnapshot {
    pub project_open: bool,
    pub scene_entry_count: usize,
    pub selected_node_id: Option<u64>,
    pub selected_node_name: Option<String>,
    pub inspector_translation: Option<[String; 3]>,
    pub inspector_scale: Option<[String; 3]>,
    pub scene_nodes: Vec<zircon_runtime::scene::NodeRecord>,
}

/// Runs normal editor bindings against the single `ProjectAuthority` generation opened by the
/// application composition. It does not interpret a binding as a filesystem mutation or reopen a
/// second project authority.
pub(crate) fn execute_project_automation(
    project_root: &ResolvedProjectPath,
    request: &EditorProjectAutomationRequest,
) -> Result<EditorProjectAutomationReport, Box<dyn Error>> {
    request.validate()?;

    let composition = EditorApplicationComposition::open_resolved_project(project_root.clone())
        .map_err(|error| {
            io::Error::other(format!(
                "could not open project '{}': {}",
                project_root.display_path().display(),
                project_root.display_diagnostic(error)
            ))
        })?;
    let (project_identity, manifest_identity, scene_uri) = {
        let opened_project = composition.prepared_project();
        let manifest = opened_project.manifest();
        (
            manifest.name.clone(),
            format!("{}@v{}", manifest.name, manifest.format_version),
            manifest.default_scene.to_string(),
        )
    };
    let retained_result = composition
        .run_retained_host_automation(&request.bindings)
        .map_err(|error| {
            io::Error::other(format!(
                "retained-host automation failed: {}",
                project_root.display_diagnostic(error)
            ))
        })?;
    require_healthy_project_for_automation(
        "Project opened: retained-host project",
        retained_result.project_info.asset_count,
        retained_result.project_info.ready_asset_count,
        retained_result.project_info.failed_asset_count,
    )?;
    let opened_project_inspection_generation =
        Some(retained_result.opened_project_inspection_generation);
    let automation_result: Result<_, Box<dyn Error>> = (|| {
        let editor_snapshot = retained_result.editor_snapshot;
        let scene_nodes = retained_result.scene_nodes;

        let selected_scene_entry = editor_snapshot
            .scene_entries
            .iter()
            .find(|entry| editor_snapshot.scene_entries.is_selected(entry.entity));
        let selected_node = selected_scene_entry
            .and_then(|entry| scene_nodes.iter().find(|node| node.id == entry.entity));
        let selected_model_resource_id = selected_node
            .and_then(|node| node.mesh.as_ref())
            .map(|mesh| mesh.model.id().to_string());
        let selected_material_resource_id = selected_node
            .and_then(|node| node.mesh.as_ref())
            .map(|mesh| mesh.material.id().to_string());
        let snapshot = EditorProjectAutomationSnapshot {
            project_open: editor_snapshot.project_open,
            scene_entry_count: editor_snapshot.scene_entries.len(),
            selected_node_id: selected_scene_entry.map(|entry| entry.entity),
            selected_node_name: selected_scene_entry.map(|entry| entry.display_name.clone()),
            inspector_translation: editor_snapshot
                .inspector
                .as_ref()
                .map(|inspector| inspector.translation.clone()),
            inspector_scale: editor_snapshot
                .inspector
                .as_ref()
                .map(|inspector| inspector.scale.clone()),
            scene_nodes,
        };
        let project_path = project_automation_report_path(Path::new(&editor_snapshot.project_path));
        if project_path.is_empty() {
            return Err(io::Error::other(
                "project-scoped editor automation completed without an opened project path",
            )
            .into());
        }

        let report = EditorProjectAutomationReport {
            project_path,
            project_identity,
            manifest_identity,
            scene_uri,
            selected_model_resource_id,
            selected_material_resource_id,
            opened_project_inspection_generation,
            records: retained_result.records,
            snapshot,
        };
        Ok(report)
    })();
    automation_result
}

fn project_automation_report_path(project_path: impl AsRef<Path>) -> String {
    let project_path = project_path.as_ref();
    if let (Ok(project_root), Ok(current_directory)) = (
        ProjectPaths::resolve_existing(project_path),
        std::env::current_dir().and_then(ProjectPaths::resolve_existing),
    ) {
        if project_root.operation_path() == current_directory.operation_path() {
            return ".".to_owned();
        }
    }

    ProjectPaths::display_path(project_path)
        .to_string_lossy()
        .into_owned()
}

fn require_healthy_project_for_automation(
    status_message: &str,
    asset_count: usize,
    ready_asset_count: usize,
    failed_asset_count: usize,
) -> Result<(), io::Error> {
    if status_message.starts_with("Project opened:")
        && failed_asset_count == 0
        && ready_asset_count == asset_count
    {
        return Ok(());
    }

    Err(io::Error::other(format!(
        "project-scoped editor automation requires a non-degraded project open: status={status_message:?} assets={asset_count} ready={ready_asset_count} failed={failed_asset_count}"
    )))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(windows)]
    use std::path::Path;

    use zircon_runtime::asset::project::ProjectPaths;

    use super::{
        normalize_resolved_project_automation_root, project_automation_report_path,
        require_healthy_project_for_automation, resolve_project_automation_input_path,
        EditorProjectAutomationReport, EditorProjectAutomationRequest,
        EditorProjectAutomationSnapshot,
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
        let source = include_str!("project_automation.rs");
        let mut offset = 0;
        for needle in [
            "EditorApplicationComposition::open_resolved_project(project_root.clone())",
            ".map_err(|error| {",
            "project_root.display_diagnostic(error)",
            "composition.run_retained_host_automation(&request.bindings)",
            "let report = EditorProjectAutomationReport",
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
        let source = include_str!("../editor.rs");
        let mut offset = 0;
        for needle in [
            "EditorLaunchRoute::Commandlet(request) => {",
            "let commandlet_host = project_automation::EditorProjectAutomationCommandletHost;",
            "run_commandlet_with_host(request, &commandlet_host)",
            "println!(\"{}\", serde_json::to_string(&report)?);",
            "return Ok(report.exit_code().as_u8());",
        ] {
            let index = source[offset..].find(needle).unwrap_or_else(|| {
                panic!("project automation commandlet entry is missing `{needle}`")
            });
            offset += index + needle.len();
        }
    }

    #[test]
    fn project_automation_commandlet_host_resolves_typed_paths_before_opening_composition() {
        let source = include_str!("project_automation.rs");
        let mut offset = 0;
        for needle in [
            "impl CommandletHost for EditorProjectAutomationCommandletHost",
            "request.project_root().to_path_buf()",
            "request.automation_path().to_path_buf()",
            "EditorApplicationComposition::open_resolved_project(project_root.clone())",
            "composition.run_retained_host_automation(&request.bindings)",
        ] {
            let index = source[offset..].find(needle).unwrap_or_else(|| {
                panic!("project automation commandlet host is missing `{needle}`")
            });
            offset += index + needle.len();
        }
    }

    #[test]
    fn empty_project_automation_request_is_rejected_before_project_open() {
        let request = EditorProjectAutomationRequest { bindings: vec![] };

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
        let error =
            resolve_project_automation_input_path(PathBuf::from(r"C:automation.json"), "file")
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
        let report = EditorProjectAutomationReport {
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
        };

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
        let authoring: EditorProjectAutomationRequest =
            serde_json::from_str(include_str!(concat!(
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
}
