use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};

use zircon_runtime::asset::project::{ProjectActivationIdentity, ProjectManager};
use zircon_runtime::asset::{AssetUri, AssetUuid};
use zircon_runtime::core::resource::ResourceScheme;
use zircon_runtime::core::CoreError;

use crate::core::asset::{EditorAssetReimportResult, EditorAssetReimportTicket};
use crate::core::jobs::{
    EditorJob, EditorJobSpec, JobCategory, JobContext, JobError, JobPriority, MutexGroup,
};
use crate::ui::host::runtime_services::EditorProjectAssetRuntimeAccess;

use super::project_mutation_guard::mutation_error;
use super::DefaultEditorAssetManager;

const PROJECT_ASSET_MUTATION_MUTEX: &str = "project_asset_mutation";

struct ProjectAssetReimportJob {
    access: EditorProjectAssetRuntimeAccess,
    projection: DefaultEditorAssetManager,
    activation: ProjectActivationIdentity,
    target_uuid: AssetUuid,
}

impl EditorJob for ProjectAssetReimportJob {
    type Output = EditorAssetReimportResult;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        context.report_progress(0, 1, format!("Reimporting asset {}", self.target_uuid));
        let (source, status) = match catch_unwind(AssertUnwindSafe(|| {
            if self.projection.project_mutation_activation()? != self.activation {
                return Err(mutation_error(
                    "reimport belongs to an inactive Editor project",
                ));
            }
            let manager = self.access.project_asset_manager()?;
            let (project, _) = manager
                .current_project_generation_snapshot()
                .ok_or_else(|| mutation_error("reimport requires an active runtime project"))?
                .into_parts();
            if project.activation_identity() != &self.activation {
                return Err(mutation_error(
                    "reimport belongs to an inactive runtime project",
                ));
            }
            let (source, source_uuid) = reimport_source(&project, self.target_uuid)?;
            // Targeted import always invokes the importer, even if source bytes and settings
            // are unchanged. Keep its original durable publication and activation fences.
            let status = manager
                .reimport_project_source_for_project(
                    &self.activation,
                    source_uuid,
                    &source.to_string(),
                )?
                .ok_or_else(|| {
                    mutation_error(
                        "reimport source is no longer registered; refresh the asset browser",
                    )
                })?;
            drop(manager);
            self.projection.refresh_from_runtime_project()?;
            Ok::<_, CoreError>((source, Some(status)))
        })) {
            Ok(result) => result.map_err(JobError::failed),
            Err(payload) => Err(JobError::Panicked(panic_message(payload))),
        }?;
        context.report_progress(1, 1, format!("Reimported asset {}", self.target_uuid));
        Ok(EditorAssetReimportResult::new(
            self.target_uuid,
            source,
            status,
        ))
    }
}

impl DefaultEditorAssetManager {
    pub fn submit_project_asset_reimport(
        &self,
        target_uuid: AssetUuid,
    ) -> Result<EditorAssetReimportTicket, CoreError> {
        let access = self.project_asset_manager.as_ref().ok_or_else(|| {
            mutation_error("project asset reimport requires the runtime asset manager")
        })?;
        let activation = self.project_mutation_activation()?;
        {
            let manager = access.project_asset_manager()?;
            let (project, _) = manager
                .current_project_generation_snapshot()
                .ok_or_else(|| mutation_error("reimport requires an active runtime project"))?
                .into_parts();
            if project.activation_identity() != &activation {
                return Err(mutation_error(
                    "reimport belongs to an inactive runtime project",
                ));
            }
            reimport_source(&project, target_uuid)?;
        }
        let jobs = self
            .preview_jobs
            .as_ref()
            .ok_or_else(|| CoreError::ServiceUnavailable("EditorJobSystem".to_owned()))?;
        let mutex = MutexGroup::parse(PROJECT_ASSET_MUTATION_MUTEX)
            .map_err(|error| mutation_error(error.to_string()))?;
        let ticket = jobs
            .submit(
                EditorJobSpec::new(format!("Reimport asset {target_uuid}"), JobCategory::Import)
                    .with_priority(JobPriority::Interactive)
                    .with_mutex_group(mutex),
                ProjectAssetReimportJob {
                    access: access.clone(),
                    projection: self.clone(),
                    activation,
                    target_uuid,
                },
            )
            .map_err(|error| CoreError::ServiceUnavailable(error.to_string()))?;
        Ok(EditorAssetReimportTicket::new(ticket, target_uuid))
    }
}

fn reimport_source(
    project: &ProjectManager,
    target_uuid: AssetUuid,
) -> Result<(AssetUri, AssetUuid), CoreError> {
    let entry = project
        .asset_registry()
        .entry_by_uuid(target_uuid)
        .ok_or_else(|| {
            mutation_error(format!(
                "asset {target_uuid} is no longer registered; refresh the asset browser"
            ))
        })?;
    if entry.path().scheme() != ResourceScheme::Res {
        return Err(mutation_error(
            "reimport requires a writable project source",
        ));
    }
    let source = AssetUri::new(entry.path().scheme(), entry.path().path(), None)
        .map_err(|error| mutation_error(error.to_string()))?;
    let source_uuid = project
        .asset_registry()
        .entry_by_path(&source)
        .map(|entry| entry.uuid())
        .ok_or_else(|| {
            mutation_error(
                "reimport source owner is no longer registered; refresh the asset browser",
            )
        })?;
    Ok((source, source_uuid))
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string project asset reimport panic payload".to_owned()
    }
}

#[cfg(test)]
#[path = "tests/project_reimport.rs"]
mod tests;
