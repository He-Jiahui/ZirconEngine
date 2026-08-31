mod authority;
mod created_project;
mod error;
mod filesystem;
mod new_project_draft;
mod new_project_template;
mod opened_project;
mod preflight_composition;
mod preflight_manifest_reader;
mod project_preflight;
mod project_probe;
mod recent_project_entry;
mod recent_project_validation;
mod scene_document;
mod scene_load_job;

pub use authority::ProjectAuthority;
pub use created_project::CreatedProject;
pub use error::ProjectAuthorityError;
pub use new_project_draft::NewProjectDraft;
pub use new_project_template::NewProjectTemplate;
pub use opened_project::OpenedProject;
pub(crate) use preflight_composition::ProjectPreflightCompositionPlan;
pub use preflight_composition::ProjectPreflightCompositionProfile;
pub use project_preflight::{
    ProjectManifestMigrationAction, ProjectManifestMigrationDecision, ProjectManifestMigrationPlan,
    ProjectPreflightReceipt, ProjectPreflightRevalidation,
};
pub use project_probe::ProjectProbe;
pub use recent_project_entry::RecentProjectEntry;
pub use recent_project_validation::RecentProjectValidation;
pub use scene_document::{ProjectSceneDocument, SceneCreateRequest, SceneOpenRequest};
pub use scene_load_job::ProjectSceneLoadTicket;
pub use zircon_runtime_interface::project::ProjectManifestDigest;

#[cfg(test)]
mod tests;
