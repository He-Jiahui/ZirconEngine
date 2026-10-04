use std::{
    env,
    error::Error,
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use zircon_editor::{
    core::{
        commandlet::{AuthoringAutomationCommandletRequest, CommandletHost},
        editor_event::EditorEventRecord,
        project::ProjectAuthority,
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
    #[serde(default)]
    pub bindings: Vec<EditorUiBinding>,
    #[serde(default, rename = "productWorkbenchCapture")]
    pub product_workbench_capture: Option<EditorProductWorkbenchCaptureRequest>,
}

/// Product snapshot and native evidence operations run by the App that owns the live Core and
/// authentic runtime BuildSet. An operation can request either or both products.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct EditorProductWorkbenchCaptureRequest {
    pub repo_root: PathBuf,
    #[serde(default)]
    pub snapshots_output_path: Option<PathBuf>,
    #[serde(default)]
    pub capture_zui_visual_evidence: bool,
}

impl EditorProductWorkbenchCaptureRequest {
    fn validate(&self) -> Result<(), io::Error> {
        if self.snapshots_output_path.is_none() && !self.capture_zui_visual_evidence {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "product workbench capture requires a snapshot output path, native visual evidence, or both",
            ));
        }
        if !self.repo_root.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "product workbench capture repoRoot must be absolute",
            ));
        }
        if self
            .snapshots_output_path
            .as_ref()
            .is_some_and(|path| path.as_os_str().is_empty() || path.file_name().is_none())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "product workbench snapshotsOutputPath must name a file",
            ));
        }
        Ok(())
    }
}

impl EditorProjectAutomationRequest {
    fn validate(&self) -> Result<(), io::Error> {
        if let Some(capture) = &self.product_workbench_capture {
            if !self.bindings.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "product workbench capture requests cannot include authoring UI bindings",
                ));
            }
            capture.validate()?;
        } else if self.bindings.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "project-scoped editor automation requires at least one UI binding",
            ));
        }

        Ok(())
    }
}

/// Process host for the commandlet that drives normal retained-host bindings and App-owned
/// product workbench capture operations.
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

const WORKBENCH_REVIEW_PROJECT_ROOT_ENV: &str = "ZIRCON_EDITOR_WORKBENCH_REVIEW_PROJECT_ROOT";

/// Supplies the capture builder's managed-project input from the same resolved identity as the
/// commandlet's `--project`. A caller-provided value is accepted only when it resolves to that
/// physical project; an installed value is restored when the capture operation finishes.
struct WorkbenchReviewProjectRoot {
    previous: Option<OsString>,
}

impl WorkbenchReviewProjectRoot {
    fn bind(project_root: &ResolvedProjectPath) -> Result<Self, io::Error> {
        let previous = env::var_os(WORKBENCH_REVIEW_PROJECT_ROOT_ENV);
        if let Some(value) = &previous {
            let configured_input = resolve_project_automation_input_path(
                PathBuf::from(value),
                "workbench review project root",
            )?;
            let configured = normalize_resolved_project_automation_root(configured_input)?;
            if configured.operation_path() != project_root.operation_path() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "{WORKBENCH_REVIEW_PROJECT_ROOT_ENV} resolves to '{}' but --project resolves to '{}'",
                        configured.display_path().display(),
                        project_root.display_path().display(),
                    ),
                ));
            }
        }

        env::set_var(
            WORKBENCH_REVIEW_PROJECT_ROOT_ENV,
            project_root.operation_path(),
        );
        Ok(Self { previous })
    }
}

impl Drop for WorkbenchReviewProjectRoot {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.take() {
            env::set_var(WORKBENCH_REVIEW_PROJECT_ROOT_ENV, previous);
        } else {
            env::remove_var(WORKBENCH_REVIEW_PROJECT_ROOT_ENV);
        }
    }
}

fn resolve_product_capture_repo_root(path: &Path) -> Result<PathBuf, io::Error> {
    let resolved = ProjectPaths::resolve_existing(path).map_err(|error| {
        io::Error::other(format!(
            "could not resolve product capture repoRoot '{}': {error}",
            ProjectPaths::display_path(path).display(),
        ))
    })?;
    if !resolved.operation_path().is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "product capture repoRoot is not a directory: {}",
                resolved.display_path().display(),
            ),
        ));
    }
    Ok(resolved.operation_path().to_path_buf())
}

fn resolve_product_capture_output_path(path: &Path) -> Result<PathBuf, io::Error> {
    if path.as_os_str().is_empty() || path.file_name().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "product workbench snapshotsOutputPath must name a file",
        ));
    }
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

/// Report variants retain the original authoring automation JSON shape while adding a
/// separate factual receipt for App-authenticated product capture operations.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub(crate) enum EditorProjectAutomationReport {
    Authoring(EditorProjectAuthoringAutomationReport),
    ProductWorkbenchCapture(EditorProductWorkbenchCaptureReport),
}

/// Structured evidence from applying a normal binding sequence to one opened project.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct EditorProjectAuthoringAutomationReport {
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

/// Capture receipt records only values supplied or returned by the live App composition.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EditorProductWorkbenchCaptureReport {
    pub operation: &'static str,
    pub project_path: String,
    pub project_identity: String,
    pub manifest_identity: String,
    pub scene_uri: String,
    pub app_preflighted_build_set_id: String,
    pub snapshots: Option<ProductWorkbenchSnapshotsReceipt>,
    pub zui_visual_evidence: Option<ZuiVisualEvidenceReceipt>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProductWorkbenchSnapshotsReceipt {
    pub output_path: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ZuiVisualEvidenceReceipt {
    pub report_path: String,
    pub captured: usize,
    pub failed: usize,
    pub pending: usize,
    pub ready: bool,
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

/// Runs normal editor bindings against the generation admitted by the retained host. The
/// metadata preflight is data-only and never materializes a runtime project before admission.
pub(crate) fn execute_project_automation(
    project_root: &ResolvedProjectPath,
    request: &EditorProjectAutomationRequest,
) -> Result<EditorProjectAutomationReport, Box<dyn Error>> {
    request.validate()?;

    let preflight = ProjectAuthority::default()
        .preflight_resolved_project(project_root)
        .map_err(|error| {
            io::Error::other(format!(
                "could not preflight project '{}': {}",
                project_root.display_path().display(),
                project_root.display_diagnostic(error)
            ))
        })?;
    let summary = preflight.summary();
    let project_identity = summary.name.clone();
    let manifest_identity = format!("{}@v{}", summary.name, summary.format_version);
    let scene_uri = summary.default_scene.clone();

    if let Some(capture_request) = &request.product_workbench_capture {
        let _review_project_root = WorkbenchReviewProjectRoot::bind(project_root)?;
        let repo_root = resolve_product_capture_repo_root(&capture_request.repo_root)?;
        let snapshots_output_path = capture_request
            .snapshots_output_path
            .as_deref()
            .map(resolve_product_capture_output_path)
            .transpose()?;
        let composition = EditorApplicationComposition::open_resolved_project(project_root.clone())
            .map_err(|error| {
                io::Error::other(format!(
                    "could not open project '{}' for product capture: {}",
                    project_root.display_path().display(),
                    project_root.display_diagnostic(error)
                ))
            })?;
        let capture_result = composition.run_product_workbench_capture(
            &repo_root,
            snapshots_output_path.as_deref(),
            capture_request.capture_zui_visual_evidence,
        )?;
        let snapshots =
            capture_result
                .snapshots_output_path
                .map(|path| ProductWorkbenchSnapshotsReceipt {
                    output_path: path.to_string_lossy().into_owned(),
                });
        let zui_visual_evidence =
            capture_result
                .visual_evidence
                .map(|summary| ZuiVisualEvidenceReceipt {
                    report_path: summary.report_path().to_string_lossy().into_owned(),
                    captured: summary.captured(),
                    failed: summary.failed(),
                    pending: summary.pending(),
                    ready: summary.is_ready(),
                });
        return Ok(EditorProjectAutomationReport::ProductWorkbenchCapture(
            EditorProductWorkbenchCaptureReport {
                operation: "productWorkbenchCapture",
                project_path: project_root.display_path().to_string_lossy().into_owned(),
                project_identity,
                manifest_identity,
                scene_uri,
                app_preflighted_build_set_id: capture_result.build_set_id,
                snapshots,
                zui_visual_evidence,
            },
        ));
    }

    let composition = EditorApplicationComposition::open_resolved_project(project_root.clone())
        .map_err(|error| {
            io::Error::other(format!(
                "could not open project '{}': {}",
                project_root.display_path().display(),
                project_root.display_diagnostic(error)
            ))
        })?;
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

        let report = EditorProjectAuthoringAutomationReport {
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
        Ok(EditorProjectAutomationReport::Authoring(report))
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
#[path = "tests/project_automation.rs"]
mod tests;
