use crate::core::jobs::{EditorJob, JobContext, JobError};
use crate::core::project::{
    NewProjectDraft, ProjectAuthority, ProjectAuthorityError, ProjectProbe,
};

pub(super) struct WelcomeProjectProbeJob {
    pub(super) draft: NewProjectDraft,
}

pub(super) struct WelcomeProjectProbeResult {
    pub(super) creation_validation: String,
    pub(super) open_probe: Result<ProjectProbe, ProjectAuthorityError>,
}

impl EditorJob for WelcomeProjectProbeJob {
    type Output = WelcomeProjectProbeResult;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        self.run_with(
            &context,
            |draft| {
                draft
                    .validate_for_creation()
                    .map(|_| String::new())
                    .unwrap_or_else(|error| error.to_string())
            },
            |draft| ProjectAuthority::default().probe_draft(draft),
        )
    }
}

impl WelcomeProjectProbeJob {
    pub(super) fn run_with<V, P>(
        &self,
        context: &JobContext,
        validate_creation: V,
        probe_existing: P,
    ) -> Result<WelcomeProjectProbeResult, JobError>
    where
        V: FnOnce(&NewProjectDraft) -> String,
        P: FnOnce(&NewProjectDraft) -> Result<ProjectProbe, ProjectAuthorityError>,
    {
        context.check_cancelled()?;
        let creation_validation = validate_creation(&self.draft);
        context.check_cancelled()?;
        let open_probe = probe_existing(&self.draft);
        context.check_cancelled()?;
        Ok(WelcomeProjectProbeResult {
            creation_validation,
            open_probe,
        })
    }
}
