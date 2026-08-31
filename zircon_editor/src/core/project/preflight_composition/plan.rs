use zircon_runtime::asset::project::ProjectScriptManifest;
use zircon_runtime::core::framework::project::ProjectPluginManifest;

use super::ProjectPreflightCompositionProfile;

/// Immutable, data-only capability policy prepared before any project-derived code is loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProjectPreflightCompositionPlan {
    profile: ProjectPreflightCompositionProfile,
    approved_project_plugins: ProjectPluginManifest,
    approved_project_scripts: ProjectScriptManifest,
    allows_project_scripts: bool,
    allows_native_extensions: bool,
    allows_scene_restore: bool,
}

impl ProjectPreflightCompositionPlan {
    pub(crate) fn compile(
        profile: ProjectPreflightCompositionProfile,
        project_plugins: &ProjectPluginManifest,
        project_scripts: &ProjectScriptManifest,
    ) -> Self {
        match profile {
            ProjectPreflightCompositionProfile::Normal => Self {
                profile,
                approved_project_plugins: project_plugins.clone(),
                approved_project_scripts: project_scripts.clone(),
                allows_project_scripts: true,
                allows_native_extensions: true,
                allows_scene_restore: true,
            },
            ProjectPreflightCompositionProfile::Safe
            | ProjectPreflightCompositionProfile::Recovery => {
                Self::without_project_derived_capabilities(profile)
            }
        }
    }

    /// Holds no project-derived inputs while migration keeps this receipt out of activation.
    pub(crate) fn without_project_derived_capabilities(
        profile: ProjectPreflightCompositionProfile,
    ) -> Self {
        Self {
            profile,
            approved_project_plugins: ProjectPluginManifest::default(),
            approved_project_scripts: ProjectScriptManifest::default(),
            allows_project_scripts: false,
            allows_native_extensions: false,
            allows_scene_restore: false,
        }
    }

    pub(crate) const fn profile(&self) -> ProjectPreflightCompositionProfile {
        self.profile
    }

    pub(crate) fn approved_project_plugins(&self) -> &ProjectPluginManifest {
        &self.approved_project_plugins
    }

    /// Static script inputs that the later materializer may consume for this admission attempt.
    pub(crate) fn approved_project_scripts(&self) -> &ProjectScriptManifest {
        &self.approved_project_scripts
    }

    pub(crate) const fn allows_project_scripts(&self) -> bool {
        self.allows_project_scripts
    }

    pub(crate) const fn allows_native_extensions(&self) -> bool {
        self.allows_native_extensions
    }

    pub(crate) const fn allows_scene_restore(&self) -> bool {
        self.allows_scene_restore
    }
}
