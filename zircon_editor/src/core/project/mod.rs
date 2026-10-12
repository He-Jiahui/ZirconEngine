#[path = "authority/mod.rs"]
mod authority;
#[path = "created_project.rs"]
mod created_project;
#[path = "error.rs"]
mod error;
#[path = "filesystem.rs"]
mod filesystem;
#[path = "new_project_draft.rs"]
mod new_project_draft;
#[path = "opened_project.rs"]
mod opened_project;
#[path = "preflight_composition/mod.rs"]
mod preflight_composition;
#[path = "preflight_manifest_reader.rs"]
mod preflight_manifest_reader;
#[path = "project_launch_preflight.rs"]
mod project_launch_preflight;
#[path = "project_preflight/mod.rs"]
mod project_preflight;
#[path = "project_probe.rs"]
mod project_probe;
#[path = "recent_project_entry.rs"]
mod recent_project_entry;
#[path = "recent_project_validation.rs"]
mod recent_project_validation;
#[path = "scene_document.rs"]
mod scene_document;
#[path = "scene_load_job.rs"]
mod scene_load_job;

pub use authority::ProjectAuthority;
pub use created_project::CreatedProject;
pub use error::ProjectAuthorityError;
pub use new_project_draft::NewProjectDraft;
pub use opened_project::OpenedProject;
pub(crate) use preflight_composition::ProjectPreflightCompositionPlan;
pub use preflight_composition::ProjectPreflightCompositionProfile;
pub use project_launch_preflight::ProjectLaunchPreflight;
pub(crate) use project_launch_preflight::ProjectLaunchPreflightTarget;
pub use project_preflight::{
    ProjectManifestMigrationAction, ProjectManifestMigrationDecision, ProjectManifestMigrationPlan,
    ProjectPreflightReceipt, ProjectPreflightRevalidation,
};
pub use project_probe::ProjectProbe;
pub use recent_project_entry::RecentProjectEntry;
pub use recent_project_validation::RecentProjectValidation;
pub use scene_document::{ProjectSceneDocument, SceneCreateRequest, SceneOpenRequest};
pub use scene_load_job::ProjectSceneLoadTicket;
pub use zircon_runtime_interface::project::{
    ProjectCreationProvenance, ProjectManifestDigest, ProjectTemplateId,
};

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
