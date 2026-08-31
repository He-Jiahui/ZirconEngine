use std::fmt;
use std::time::Instant;

use zircon_runtime::asset::{project::ProjectManager, AssetUri};

use crate::core::jobs::{
    EditorJob, EditorJobSpec, EditorJobSystem, JobCategory, JobContext, JobError, JobId,
    JobPriority, JobSubmitError, JobTicket,
};

use super::{ProjectAuthority, ProjectSceneDocument, SceneOpenRequest};

pub struct ProjectSceneLoadTicket {
    ticket: JobTicket<ProjectSceneDocument>,
    scene_uri: AssetUri,
}

impl ProjectSceneLoadTicket {
    fn new(ticket: JobTicket<ProjectSceneDocument>, scene_uri: AssetUri) -> Self {
        Self { ticket, scene_uri }
    }

    pub fn id(&self) -> JobId {
        self.ticket.id()
    }

    pub fn scene_uri(&self) -> &AssetUri {
        &self.scene_uri
    }

    pub fn try_take(&self) -> Option<Result<ProjectSceneDocument, JobError>> {
        self.ticket.try_take()
    }

    pub fn wait_until(&self, deadline: Instant) -> Option<Result<ProjectSceneDocument, JobError>> {
        self.ticket.wait_until(deadline)
    }
}

impl fmt::Debug for ProjectSceneLoadTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProjectSceneLoadTicket")
            .field("scene_uri", &self.scene_uri)
            .field("job_id", &self.id())
            .finish()
    }
}

struct ProjectSceneLoadJob {
    project: ProjectManager,
    request: SceneOpenRequest,
}

impl EditorJob for ProjectSceneLoadJob {
    type Output = ProjectSceneDocument;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        context.report_progress(0, 1, format!("Loading scene {}", self.request.scene_uri()));
        zircon_runtime::profile_scope!("editor", "project", "scene_load_job");
        let document = ProjectAuthority::default()
            .open_scene(&self.project, self.request)
            .map_err(JobError::failed)?;
        context.check_cancelled()?;
        context.report_progress(1, 1, format!("Loaded scene {}", document.scene_uri()));
        Ok(document)
    }
}

impl ProjectAuthority {
    /// Submits validated project-scene I/O and deserialization without touching editor UI state.
    pub fn submit_scene_open(
        &self,
        jobs: &EditorJobSystem,
        project: ProjectManager,
        request: SceneOpenRequest,
    ) -> Result<ProjectSceneLoadTicket, JobSubmitError> {
        let scene_uri = request.scene_uri().clone();
        let ticket = jobs.submit(
            EditorJobSpec::new(format!("Load scene {scene_uri}"), JobCategory::Index)
                .with_priority(JobPriority::Interactive),
            ProjectSceneLoadJob { project, request },
        )?;
        Ok(ProjectSceneLoadTicket::new(ticket, scene_uri))
    }
}
