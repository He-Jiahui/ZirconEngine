use std::path::Path;

use zircon_runtime::asset::project::ResolvedProjectPath;
use zircon_runtime_interface::project::{
    assess_project_engine_compatibility, CanonicalDescriptorIdentity, ProjectEngineCompatibility,
    ProjectEngineCompatibilityError, ProjectEngineVersion, ProjectIdentity, ProjectManifestDigest,
    ProjectManifestSummary,
};

use crate::core::project::{ProjectAuthorityError, ProjectPreflightCompositionPlan};

use super::ProjectManifestMigrationDecision;

/// Data-only project evidence produced before session admission or project-code activation.
///
/// The receipt contains no `ProjectManager`, plugin, runtime, or filesystem-mutation capability.
/// Its caller must still apply engine/BuildSet/trust/recovery policy and acquire the project
/// admission lease before opening the project runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectPreflightReceipt {
    resolved_project_path: ResolvedProjectPath,
    canonical_descriptor: CanonicalDescriptorIdentity,
    project_identity: Option<ProjectIdentity>,
    summary: ProjectManifestSummary,
    composition: ProjectPreflightCompositionPlan,
    manifest_migration: ProjectManifestMigrationDecision,
    manifest_digest: ProjectManifestDigest,
}

impl ProjectPreflightReceipt {
    pub(in crate::core::project) fn new(
        resolved_project_path: ResolvedProjectPath,
        summary: ProjectManifestSummary,
        composition: ProjectPreflightCompositionPlan,
        manifest_migration: ProjectManifestMigrationDecision,
        manifest_digest: ProjectManifestDigest,
    ) -> Result<Self, ProjectAuthorityError> {
        let canonical_descriptor =
            CanonicalDescriptorIdentity::new(resolved_project_path.operation_path().to_path_buf())?;
        let project_identity = if manifest_migration.blocks_activation() {
            None
        } else {
            let project_guid = summary
                .project_guid
                .ok_or(ProjectAuthorityError::CurrentManifestMissingProjectGuid)?;
            Some(ProjectIdentity::new(
                canonical_descriptor.clone(),
                project_guid,
                manifest_digest,
            ))
        };
        Ok(Self {
            resolved_project_path,
            canonical_descriptor,
            project_identity,
            summary,
            composition,
            manifest_migration,
            manifest_digest,
        })
    }

    pub fn root(&self) -> &Path {
        self.resolved_project_path.operation_path()
    }

    /// Canonical descriptor identity; display paths are intentionally excluded from this value.
    pub fn canonical_descriptor(&self) -> &CanonicalDescriptorIdentity {
        &self.canonical_descriptor
    }

    /// Current manifests have complete typed identity; migration candidates cannot be admitted.
    pub fn project_identity(&self) -> Option<&ProjectIdentity> {
        self.project_identity.as_ref()
    }

    pub(crate) fn resolved_project_path(&self) -> &ResolvedProjectPath {
        &self.resolved_project_path
    }

    pub fn summary(&self) -> &ProjectManifestSummary {
        &self.summary
    }

    /// Contains static policy only; activation code may consume it only after admission.
    pub(crate) fn composition(&self) -> &ProjectPreflightCompositionPlan {
        &self.composition
    }

    pub const fn manifest_migration(&self) -> ProjectManifestMigrationDecision {
        self.manifest_migration
    }

    pub const fn manifest_digest(&self) -> ProjectManifestDigest {
        self.manifest_digest
    }

    /// Evaluates only the manifest's engine semantic-version requirement without activation.
    ///
    /// BuildSet and provider requirements remain separate admission dimensions until their
    /// project-manifest schema is available; a compatible engine range is not full admission.
    pub fn evaluate_engine_compatibility(
        &self,
        running_engine: &ProjectEngineVersion,
    ) -> Result<ProjectEngineCompatibility, ProjectEngineCompatibilityError> {
        assess_project_engine_compatibility(
            self.summary.engine_version_req.as_deref(),
            running_engine,
        )
    }
}
