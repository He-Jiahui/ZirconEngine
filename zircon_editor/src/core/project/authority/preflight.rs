use std::path::Path;
use std::sync::Arc;

use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths, ResolvedProjectPath};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime_interface::project::{
    render_project_template, ProjectEngineVersion, ProjectLaunchIntent, ProjectLaunchProfile,
    ProjectLaunchTarget, ProjectTemplatePackError,
};

use super::super::preflight_manifest_reader::inspect_project_manifest;
use super::ProjectAuthority;
use crate::core::project::{
    NewProjectDraft, ProjectAuthorityError, ProjectLaunchPreflight, ProjectLaunchPreflightTarget,
    ProjectManifestMigrationDecision, ProjectPreflightCompositionPlan,
    ProjectPreflightCompositionProfile, ProjectPreflightReceipt, ProjectPreflightRevalidation,
    ProjectProbe,
};

impl ProjectAuthority {
    /// Freezes project-derived provider inputs before App composes the product host.
    pub fn preflight_project_launch(
        &self,
        intent: ProjectLaunchIntent,
    ) -> Result<ProjectLaunchPreflight, ProjectAuthorityError> {
        let target = match intent.target() {
            ProjectLaunchTarget::OpenExisting { requested_path } => {
                let profile = composition_profile(intent.profile());
                let receipt =
                    self.preflight_project_with_composition_profile(requested_path, profile)?;
                validate_launch_receipt(&receipt)?;
                ProjectLaunchPreflightTarget::Existing(receipt)
            }
            ProjectLaunchTarget::CreateProject {
                project_name,
                location,
                template,
            } => {
                if !matches!(intent.profile(), ProjectLaunchProfile::Normal) {
                    return Err(ProjectAuthorityError::UnsupportedCreationProfile);
                }
                let draft = NewProjectDraft {
                    project_name: project_name.clone(),
                    location: location.to_string_lossy().into_owned(),
                    template: *template,
                };
                let target_root = draft.validate_for_creation()?;
                let rendered = Arc::new(render_project_template(*template, project_name)?);
                let manifest_entry = rendered
                    .entries
                    .iter()
                    .find(|entry| entry.path.as_str() == "zircon-project.toml")
                    .ok_or(ProjectTemplatePackError::MissingManifest)?;
                let manifest_source = std::str::from_utf8(&manifest_entry.bytes)
                    .map_err(|source| ProjectTemplatePackError::ManifestUtf8 { source })?;
                let manifest = ProjectManifest::from_toml_str(manifest_source)?.value;
                let composition = ProjectPreflightCompositionPlan::compile(
                    ProjectPreflightCompositionProfile::Normal,
                    &manifest.plugins,
                    &manifest.scripts,
                );
                ProjectLaunchPreflightTarget::Create {
                    target_root,
                    rendered,
                    composition,
                }
            }
        };
        Ok(ProjectLaunchPreflight::from_target(intent, target))
    }

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

fn composition_profile(profile: ProjectLaunchProfile) -> ProjectPreflightCompositionProfile {
    match profile {
        ProjectLaunchProfile::Normal => ProjectPreflightCompositionProfile::Normal,
        ProjectLaunchProfile::Safe => ProjectPreflightCompositionProfile::Safe,
        ProjectLaunchProfile::Recovery => ProjectPreflightCompositionProfile::Recovery,
    }
}

fn validate_launch_receipt(receipt: &ProjectPreflightReceipt) -> Result<(), ProjectAuthorityError> {
    if receipt.manifest_migration().blocks_activation() {
        return Err(ProjectAuthorityError::ManifestMigrationRequired);
    }
    let engine = ProjectEngineVersion::parse(env!("CARGO_PKG_VERSION"))?;
    let compatibility = receipt.evaluate_engine_compatibility(&engine)?;
    if !compatibility.is_compatible() {
        return Err(ProjectAuthorityError::IncompatibleEngine {
            disposition: compatibility.disposition(),
        });
    }
    if receipt.project_identity().is_none() {
        return Err(ProjectAuthorityError::CurrentManifestMissingProjectGuid);
    }
    if receipt.composition().profile() == ProjectPreflightCompositionProfile::Normal {
        validate_template_provider_requirements(receipt)?;
    }
    Ok(())
}

fn validate_template_provider_requirements(
    receipt: &ProjectPreflightReceipt,
) -> Result<(), ProjectAuthorityError> {
    let Some(template_receipt) = receipt.summary().template_receipt.as_ref() else {
        return Ok(());
    };
    let requirement = template_receipt
        .descriptor()
        .target_requirements()
        .iter()
        .find(|requirement| {
            requirement.target()
                == zircon_runtime_interface::runtime_build_set::ZrRuntimeModuleCompositionTargetV1::EditorHost
        })
        .ok_or(ProjectAuthorityError::TemplateEditorTargetMissing)?;
    let plugins = receipt.composition().approved_project_plugins();
    for provider in requirement.required_runtime_providers() {
        let found = plugins
            .selections
            .iter()
            .filter(|selection| {
                selection.id == provider
                    && selection.enabled
                    && selection.required
                    && selection.supports_target(RuntimeTargetMode::EditorHost)
            })
            .count();
        if found != 1 {
            return Err(ProjectAuthorityError::TemplateProviderRequirement {
                provider: provider.to_string(),
                found,
            });
        }
    }
    Ok(())
}
