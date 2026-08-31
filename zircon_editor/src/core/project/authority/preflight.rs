use std::path::Path;

use zircon_runtime::asset::project::{ProjectPaths, ResolvedProjectPath};

use super::super::preflight_manifest_reader::inspect_project_manifest;
use super::ProjectAuthority;
use crate::core::project::{
    NewProjectDraft, ProjectAuthorityError, ProjectManifestMigrationDecision,
    ProjectPreflightCompositionPlan, ProjectPreflightCompositionProfile, ProjectPreflightReceipt,
    ProjectPreflightRevalidation, ProjectProbe,
};

impl ProjectAuthority {
    pub fn probe_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<ProjectProbe, ProjectAuthorityError> {
        let preflight = self.preflight_project(path)?;
        Ok(ProjectProbe::new(
            preflight.resolved_project_path().clone(),
            preflight.summary().clone(),
        ))
    }

    /// Reads canonical project identity and manifest evidence without opening runtime project state.
    ///
    /// This is intentionally before session admission: it performs no derived-layout creation,
    /// asset indexing, plugin discovery, or runtime construction. Admission policy must decide how
    /// to handle `manifest_migration` before it permits project-derived code to execute.
    pub fn preflight_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<ProjectPreflightReceipt, ProjectAuthorityError> {
        self.preflight_project_with_composition_profile(
            path,
            ProjectPreflightCompositionProfile::Normal,
        )
    }

    /// Preflights an existing project under a composition policy selected before admission.
    pub fn preflight_project_with_composition_profile(
        &self,
        path: impl AsRef<Path>,
        composition_profile: ProjectPreflightCompositionProfile,
    ) -> Result<ProjectPreflightReceipt, ProjectAuthorityError> {
        let root = self.resolve_existing_project_root_with_identity(path)?;
        self.preflight_resolved_project_with_composition_profile(&root, composition_profile)
    }

    /// Preflights a physical project identity supplied by an upstream path boundary.
    pub fn preflight_resolved_project(
        &self,
        root: &ResolvedProjectPath,
    ) -> Result<ProjectPreflightReceipt, ProjectAuthorityError> {
        self.preflight_resolved_project_with_composition_profile(
            root,
            ProjectPreflightCompositionProfile::Normal,
        )
    }

    /// Preflights a physical project identity under a caller-selected composition policy.
    pub fn preflight_resolved_project_with_composition_profile(
        &self,
        root: &ResolvedProjectPath,
        composition_profile: ProjectPreflightCompositionProfile,
    ) -> Result<ProjectPreflightReceipt, ProjectAuthorityError> {
        super::super::filesystem::validate_canonical_existing_project_root(root.operation_path())?;
        let paths = ProjectPaths::from_resolved_root(root);
        let inspection = inspect_project_manifest(paths.manifest_path())?;
        let composition = inspection.manifest.as_ref().map_or_else(
            || {
                ProjectPreflightCompositionPlan::without_project_derived_capabilities(
                    composition_profile,
                )
            },
            |manifest| {
                ProjectPreflightCompositionPlan::compile(
                    composition_profile,
                    &manifest.plugins,
                    &manifest.scripts,
                )
            },
        );
        let manifest_migration =
            ProjectManifestMigrationDecision::from_migrated_from(inspection.migrated_from);
        ProjectPreflightReceipt::new(
            root.clone(),
            inspection.summary,
            composition,
            manifest_migration,
            inspection.digest,
        )
    }

    /// Re-reads data-only manifest evidence before admission commits to a previous preflight.
    ///
    /// The returned `Changed` state is a mandatory policy boundary: callers must not reuse a
    /// compatibility, migration, or trust decision made for the earlier manifest fingerprint.
    pub fn revalidate_preflight(
        &self,
        approved: &ProjectPreflightReceipt,
    ) -> Result<ProjectPreflightRevalidation, ProjectAuthorityError> {
        let observed = self.preflight_resolved_project_with_composition_profile(
            approved.resolved_project_path(),
            approved.composition().profile(),
        )?;
        if observed.manifest_digest() == approved.manifest_digest() {
            Ok(ProjectPreflightRevalidation::Unchanged { current: observed })
        } else {
            Ok(ProjectPreflightRevalidation::Changed {
                expected: approved.manifest_digest(),
                observed,
            })
        }
    }

    pub fn probe_draft(
        &self,
        draft: &NewProjectDraft,
    ) -> Result<ProjectProbe, ProjectAuthorityError> {
        self.probe_project(draft.project_root()?)
    }

    pub fn preflight_draft(
        &self,
        draft: &NewProjectDraft,
    ) -> Result<ProjectPreflightReceipt, ProjectAuthorityError> {
        self.preflight_project(draft.project_root()?)
    }
}
