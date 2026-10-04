use std::path::Path;
use std::sync::OnceLock;

use crate::core::project::{
    NewProjectDraft, ProjectAuthority, ProjectLaunchPreflight, ProjectLaunchPreflightTarget,
    RecentProjectEntry,
};
use crate::core::recovery::ProjectRecoveryAssessment;
use crate::ui::workbench::startup::EditorStartupSessionDocument;
use zircon_runtime_interface::project::{
    ProjectActivationOperationId, ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
    ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource, ProjectLaunchTarget,
    ProjectTemplateId,
};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

static LOCAL_PROJECT_LAUNCH_OPERATION_IDS: OnceLock<ProjectActivationOperationIdGenerator> =
    OnceLock::new();

impl EditorManager {
    /// Accepts only the BuildSet that App authenticated before this Editor host was composed.
    pub(crate) fn configure_project_runtime_build_set(
        &self,
        build_set_id: Option<ZrRuntimeBuildSetId>,
    ) {
        *self
            .project_runtime_build_set
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = build_set_id;
    }

    pub fn resolve_startup_session(&self) -> Result<EditorStartupSessionDocument, EditorError> {
        self.host.resolve_startup_session()
    }

    pub fn open_project_and_remember(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        self.execute_project_launch_intent(self.local_open_project_intent(path.as_ref())?)
    }

    /// Executes the versioned request through data-only preflight and the Editor-owned admission
    /// boundary. UI callers cannot transfer a materialized project through this API.
    pub fn execute_project_launch_intent(
        &self,
        intent: ProjectLaunchIntent,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let preflight = ProjectAuthority::default().preflight_project_launch(intent)?;
        self.execute_project_launch_preflight(preflight)
    }

    /// Executes the immutable data-only preflight produced by App or a local UI caller.
    pub(crate) fn execute_project_launch_preflight(
        &self,
        preflight: ProjectLaunchPreflight,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let (intent, target) = preflight.into_parts();
        let admission = self.session_admission_request(&intent)?;
        match target {
            ProjectLaunchPreflightTarget::Existing(receipt) => {
                if matches!(intent.profile(), ProjectLaunchProfile::Recovery) {
                    self.recover_project_and_remember_with_session(receipt, &admission)
                } else {
                    self.open_project_and_remember_with_session(receipt, &admission)
                }
            }
            ProjectLaunchPreflightTarget::Create { .. } => {
                self.create_project_and_open_with_session(target, &admission)
            }
        }
    }

    pub(super) fn preflight_existing_project_launch(
        &self,
        intent: &ProjectLaunchIntent,
        _requested_path: &Path,
    ) -> Result<crate::core::project::ProjectPreflightReceipt, EditorError> {
        let preflight = ProjectAuthority::default().preflight_project_launch(intent.clone())?;
        let (_, target) = preflight.into_parts();
        match target {
            ProjectLaunchPreflightTarget::Existing(receipt) => Ok(receipt),
            ProjectLaunchPreflightTarget::Create { .. } => Err(EditorError::Project(
                "existing project preflight received a create launch target".to_string(),
            )),
        }
    }

    pub fn create_project_and_open(
        &self,
        draft: NewProjectDraft,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let intent = ProjectLaunchIntent::create_project(
            next_local_project_launch_operation_id()?,
            ProjectLaunchSource::Welcome,
            ProjectLaunchProfile::Normal,
            draft.project_name,
            draft.location,
            draft.template,
        )
        .map_err(|error| EditorError::Project(error.to_string()))?;
        self.execute_project_launch_intent(intent)
    }

    pub fn recent_projects_snapshot(&self) -> Result<Vec<RecentProjectEntry>, EditorError> {
        self.host.recent_projects_snapshot()
    }

    pub fn forget_recent_project(&self, path: impl AsRef<Path>) -> Result<(), EditorError> {
        self.host.forget_recent_project(path)
    }

    /// Produces only recovery diagnostics. The caller cannot use this snapshot to acquire or
    /// replace a project writer lease; recovery admission rechecks under its owned OS lease.
    pub(crate) fn inspect_project_recovery(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<ProjectRecoveryAssessment, EditorError> {
        ProjectRecoveryAssessment::inspect(path.as_ref()).map_err(|error| {
            EditorError::Project(format!(
                "cannot inspect project recovery state for `{}`: {error}",
                path.as_ref().display(),
            ))
        })
    }

    pub fn update_recent_project(&self, path: impl AsRef<Path>) -> Result<(), EditorError> {
        self.host.update_recent_project(path)
    }

    pub(crate) fn show_welcome_page(&self) -> Result<(), EditorError> {
        self.host.show_welcome_page()
    }

    pub(crate) fn dismiss_welcome_page(&self) -> Result<(), EditorError> {
        self.host.dismiss_welcome_page()
    }

    pub(super) fn session_admission_request(
        &self,
        intent: &ProjectLaunchIntent,
    ) -> Result<crate::core::recovery::SessionAdmissionRequest, EditorError> {
        let build_set_id = self
            .project_runtime_build_set
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
            .ok_or_else(|| {
                EditorError::Project(
                    "project admission requires the BuildSet App authenticated during startup"
                        .to_string(),
                )
            })?;
        Ok(
            crate::core::recovery::SessionAdmissionRequest::from_launch_intent(
                intent,
                build_set_id,
            ),
        )
    }

    pub(super) fn local_open_project_intent(
        &self,
        path: &Path,
    ) -> Result<ProjectLaunchIntent, EditorError> {
        ProjectLaunchIntent::open_existing(
            next_local_project_launch_operation_id()?,
            ProjectLaunchSource::Welcome,
            ProjectLaunchProfile::Normal,
            path,
        )
        .map_err(|error| EditorError::Project(error.to_string()))
    }
}

fn next_local_project_launch_operation_id() -> Result<ProjectActivationOperationId, EditorError> {
    LOCAL_PROJECT_LAUNCH_OPERATION_IDS
        .get_or_init(|| ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new()))
        .allocate()
        .ok_or_else(|| {
            EditorError::Project("project launch operation sequence is exhausted".to_string())
        })
}

#[cfg(test)]
#[path = "tests/editor_manager_startup.rs"]
mod tests;
