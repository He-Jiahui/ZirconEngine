use std::path::Path;
use std::sync::OnceLock;

use crate::core::project::{
    NewProjectDraft, NewProjectTemplate, ProjectAuthority, ProjectPreflightCompositionProfile,
    RecentProjectEntry,
};
use crate::core::recovery::ProjectRecoveryAssessment;
use crate::ui::workbench::startup::EditorStartupSessionDocument;
use zircon_runtime_interface::project::{
    ProjectActivationOperationId, ProjectActivationOperationIdGenerator, ProjectEngineVersion,
    ProjectLaunchInstanceId, ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource,
    ProjectLaunchTarget, ProjectTemplateId,
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
        let admission = self.session_admission_request(&intent)?;
        match intent.target() {
            ProjectLaunchTarget::OpenExisting { requested_path } => {
                let preflight = self.preflight_existing_project_launch(&intent, requested_path)?;
                if matches!(intent.profile(), ProjectLaunchProfile::Recovery) {
                    self.recover_project_and_remember_with_session(preflight, &admission)
                } else {
                    self.open_project_and_remember_with_session(preflight, &admission)
                }
            }
            ProjectLaunchTarget::CreateProject {
                project_name,
                location,
                template,
            } => {
                if !matches!(intent.profile(), ProjectLaunchProfile::Normal) {
                    return Err(EditorError::Project(
                        "safe and recovery profiles can only open an existing project".to_string(),
                    ));
                }
                let template = match template {
                    ProjectTemplateId::RenderableEmpty => NewProjectTemplate::RenderableEmpty,
                };
                self.create_project_and_open_with_session(
                    NewProjectDraft {
                        project_name: project_name.clone(),
                        location: location.to_string_lossy().into_owned(),
                        template,
                    },
                    &admission,
                )
            }
        }
    }

    pub(super) fn preflight_existing_project_launch(
        &self,
        intent: &ProjectLaunchIntent,
        requested_path: &Path,
    ) -> Result<crate::core::project::ProjectPreflightReceipt, EditorError> {
        let profile = match intent.profile() {
            ProjectLaunchProfile::Normal => ProjectPreflightCompositionProfile::Normal,
            ProjectLaunchProfile::Safe => ProjectPreflightCompositionProfile::Safe,
            ProjectLaunchProfile::Recovery => ProjectPreflightCompositionProfile::Recovery,
        };
        let receipt = ProjectAuthority::default()
            .preflight_project_with_composition_profile(requested_path, profile)
            .map_err(|error| EditorError::Project(error.to_string()))?;
        if receipt.manifest_migration().blocks_activation() {
            return Err(EditorError::Project(
                "project manifest requires an explicit migration decision before activation"
                    .to_string(),
            ));
        }
        let engine = ProjectEngineVersion::parse(env!("CARGO_PKG_VERSION"))
            .map_err(|error| EditorError::Project(error.to_string()))?;
        let engine_compatibility = receipt
            .evaluate_engine_compatibility(&engine)
            .map_err(|error| EditorError::Project(error.to_string()))?;
        if !engine_compatibility.is_compatible() {
            return Err(EditorError::Project(format!(
                "project engine compatibility rejected activation: {:?}",
                engine_compatibility.disposition()
            )));
        }
        if receipt.project_identity().is_none() {
            return Err(EditorError::Project(
                "current project preflight did not produce canonical project identity".to_string(),
            ));
        }
        Ok(receipt)
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
            match draft.template {
                NewProjectTemplate::RenderableEmpty => ProjectTemplateId::RenderableEmpty,
            },
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
mod tests {
    #[test]
    fn recovery_profile_defers_authoritative_assessment_to_leased_admission() {
        let source = include_str!("editor_manager_startup.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("startup production source");
        let recovery_profile = production
            .find("ProjectLaunchProfile::Recovery")
            .expect("recovery launch branch");
        let takeover = production
            .find("self.recover_project_and_remember_with_session")
            .expect("recovery takeover dispatch");

        assert!(recovery_profile < takeover);
        assert!(!production.contains("require_recovery_profile_takeover"));
    }

    #[test]
    fn project_session_transition_recovery_decisions_remain_serialized_and_fail_closed() {
        let session = include_str!("editor_manager_project_session.rs");
        let recovery = session
            .split("pub(super) fn recover_project_and_remember_with_session")
            .nth(1)
            .expect("serialized recovery activation implementation");
        let gate = recovery
            .find("self.begin_project_session_transition()?")
            .expect("recovery activation must hold the transition gate");
        let activate = recovery
            .find("self.activate_project_from_preflight(")
            .expect("recovery activation call");
        let begin = recovery
            .find("self.begin_project_recovery_decisions(")
            .expect("recovery decisions must begin before releasing the transition gate");
        let retain = recovery
            .find("self.retain_project_session_for_recovery(error)")
            .expect("recovery coordinator failure must retain the exclusive recovery fence");

        assert!(gate < activate);
        assert!(activate < begin);
        assert!(begin < retain);
    }
}
