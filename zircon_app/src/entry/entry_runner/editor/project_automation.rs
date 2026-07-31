use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use zircon_editor::{
    core::editor_event::{EditorEventRecord, EditorEventSource},
    ui::binding::EditorUiBinding,
};

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

/// CLI input for one project-scoped, headless binding sequence.
pub(crate) struct EditorProjectAutomationCliRequest {
    pub project_root: PathBuf,
    pub request: EditorProjectAutomationRequest,
}

/// Parses the optional project automation CLI without changing the normal GUI or operation CLI.
pub(crate) fn parse_project_automation_args<I>(
    args: I,
) -> Result<Option<EditorProjectAutomationCliRequest>, Box<dyn Error>>
where
    I: IntoIterator<Item = String>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if !args.iter().any(|arg| arg == "--automation") {
        return Ok(None);
    }

    let mut args = args.into_iter();
    let mut project_root = None;
    let mut automation_path = None;
    let mut headless = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--project" => {
                if project_root.is_some() {
                    return Err("--project was provided more than once".into());
                }
                let Some(value) = args.next() else {
                    return Err("--project requires a project path".into());
                };
                if value.trim().is_empty() {
                    return Err("--project requires a non-empty project path".into());
                }
                project_root = Some(PathBuf::from(value));
            }
            "--automation" => {
                if automation_path.is_some() {
                    return Err("--automation was provided more than once".into());
                }
                let Some(value) = args.next() else {
                    return Err("--automation requires a JSON file path".into());
                };
                if value.trim().is_empty() {
                    return Err("--automation requires a non-empty JSON file path".into());
                }
                automation_path = Some(PathBuf::from(value));
            }
            "--headless" => {
                if headless {
                    return Err("--headless was provided more than once".into());
                }
                headless = true;
            }
            other => {
                return Err(
                    format!("unknown project-scoped editor automation argument `{other}`").into(),
                );
            }
        }
    }

    let Some(project_root) = project_root else {
        return Err("--automation requires --project <project-root>".into());
    };
    let Some(automation_path) = automation_path else {
        return Ok(None);
    };
    if !headless {
        return Err("--automation requires --headless".into());
    }

    let contents = fs::read_to_string(&automation_path).map_err(|error| {
        io::Error::other(format!(
            "could not read project automation file '{}': {error}",
            automation_path.display()
        ))
    })?;
    let request: EditorProjectAutomationRequest =
        serde_json::from_str(&contents).map_err(|error| {
            io::Error::other(format!(
                "could not parse project automation file '{}': {error}",
                automation_path.display()
            ))
        })?;
    request.validate()?;

    Ok(Some(EditorProjectAutomationCliRequest {
        project_root,
        request,
    }))
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
    /// Compact retained-host projection captured after the final normal binding dispatch.
    /// Keeping this derived from `editor_snapshot` lets a later process prove persisted editor
    /// state without bypassing the editor's project-open and selection paths.
    pub snapshot: EditorProjectAutomationSnapshot,
}

/// Stable subset of the retained editor projection needed by product automation evidence.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct EditorProjectAutomationSnapshot {
    pub project_open: bool,
    pub scene_entry_count: usize,
    pub selected_node_id: Option<u64>,
    pub selected_node_name: Option<String>,
    pub inspector_translation: Option<[String; 3]>,
}

/// Runs normal editor bindings against the single `ProjectAuthority` generation opened by the
/// application composition. It does not interpret a binding as a filesystem mutation or reopen a
/// second project authority.
pub(crate) fn execute_project_automation(
    project_root: impl AsRef<Path>,
    request: &EditorProjectAutomationRequest,
) -> Result<EditorProjectAutomationReport, Box<dyn Error>> {
    request.validate()?;

    let composition = EditorApplicationComposition::open_project(project_root)?;
    let opened_project = composition
        .startup_session()
        .project
        .as_ref()
        .ok_or_else(|| io::Error::other("project automation composition has no opened project"))?;
    let project_identity = opened_project.manifest.name.clone();
    let manifest_identity = format!(
        "{}@v{}",
        opened_project.manifest.name, opened_project.manifest.format_version
    );
    let scene_uri = opened_project.manifest.default_scene.to_string();
    let opened_project_inspection_generation = composition.opened_project_inspection_generation();
    let host = composition.editor_host();
    let mut records = Vec::with_capacity(request.bindings.len());
    for (index, binding) in request.bindings.iter().cloned().enumerate() {
        let binding_path = binding.native_binding();
        let record = host
            .dispatch_binding(binding, EditorEventSource::Cli)
            .map_err(|error| {
                io::Error::other(format!(
                    "project-scoped editor automation binding {index} ('{binding_path}') failed: {error}"
                ))
            })?;
        records.push(record);
    }

    let editor_snapshot = host.editor_snapshot();
    let selected_scene_entry = editor_snapshot
        .scene_entries
        .iter()
        .find(|entry| entry.selected);
    let selected_mesh = selected_scene_entry
        .and_then(|entry| opened_project.world.find_node(entry.id))
        .and_then(|node| node.mesh);
    let snapshot = EditorProjectAutomationSnapshot {
        project_open: editor_snapshot.project_open,
        scene_entry_count: editor_snapshot.scene_entries.len(),
        selected_node_id: selected_scene_entry.map(|entry| entry.id),
        selected_node_name: selected_scene_entry.map(|entry| entry.name.clone()),
        inspector_translation: editor_snapshot
            .inspector
            .as_ref()
            .map(|inspector| inspector.translation.clone()),
    };
    let project_path = editor_snapshot.project_path;
    if project_path.is_empty() {
        return Err(io::Error::other(
            "project-scoped editor automation completed without an opened project path",
        )
        .into());
    }

    Ok(EditorProjectAutomationReport {
        project_path,
        project_identity,
        manifest_identity,
        scene_uri,
        selected_model_resource_id: selected_mesh
            .as_ref()
            .map(|mesh| mesh.model.id().to_string()),
        selected_material_resource_id: selected_mesh
            .as_ref()
            .map(|mesh| mesh.material.id().to_string()),
        opened_project_inspection_generation,
        records,
        snapshot,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        parse_project_automation_args, EditorProjectAutomationReport,
        EditorProjectAutomationRequest, EditorProjectAutomationSnapshot,
    };

    #[test]
    fn empty_project_automation_request_is_rejected_before_project_open() {
        let request = EditorProjectAutomationRequest { bindings: vec![] };

        assert_eq!(
            request.validate().unwrap_err().to_string(),
            "project-scoped editor automation requires at least one UI binding"
        );
    }

    #[test]
    fn project_automation_cli_requires_project_before_reading_automation_file() {
        let error = parse_project_automation_args([
            "--automation".to_string(),
            "missing.json".to_string(),
            "--headless".to_string(),
        ])
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "--automation requires --project <project-root>"
        );
    }

    #[test]
    fn project_automation_cli_requires_headless_before_reading_automation_file() {
        let error = parse_project_automation_args([
            "--project".to_string(),
            "project-root".to_string(),
            "--automation".to_string(),
            "missing.json".to_string(),
        ])
        .unwrap_err();

        assert_eq!(error.to_string(), "--automation requires --headless");
    }

    #[test]
    fn project_automation_cli_rejects_empty_required_paths_before_reading_files() {
        for (args, expected_error) in [
            (
                [
                    "--project",
                    " ",
                    "--automation",
                    "missing.json",
                    "--headless",
                ],
                "--project requires a non-empty project path",
            ),
            (
                [
                    "--project",
                    "project-root",
                    "--automation",
                    " ",
                    "--headless",
                ],
                "--automation requires a non-empty JSON file path",
            ),
        ] {
            let error = parse_project_automation_args(args.map(str::to_string)).unwrap_err();

            assert_eq!(error.to_string(), expected_error);
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
            },
        };

        let value = serde_json::to_value(report).expect("report serialization should succeed");
        assert_eq!(value["snapshot"]["selected_node_id"], 3);
        assert_eq!(value["snapshot"]["selected_node_name"], "Cube");
        assert_eq!(value["snapshot"]["inspector_translation"][0], "42");
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
