use std::path::Path;

use zircon_runtime::plugin::{
    native::{NativePluginArtifactAuthority, NativePluginArtifactTarget},
    package_service::{resolve_project_native_plugin_admission, NativePluginAdmission},
};

use super::super::super::editor_manager::EditorManager;

impl EditorManager {
    /// Resolves selected installed packages using the App-authenticated BuildSet and host policy.
    /// Project data supplies only the selection IDs and required flags; it cannot select trust.
    pub(crate) fn native_plugin_admission_for_project(
        &self,
        project_root: &Path,
        target: NativePluginArtifactTarget,
    ) -> Result<NativePluginAdmission, String> {
        let project = self
            .host
            .current_project_snapshot_for_plugin_authority()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "native plugin authority requires an open project".to_string())?;
        if project.paths().root() != project_root {
            return Err(format!(
                "native plugin authority project root changed from {} to {}",
                project.paths().root().display(),
                project_root.display()
            ));
        }
        let build_set_id = self
            .project_runtime_build_set
            .lock()
            .map_err(|_| "project runtime BuildSet lock is poisoned".to_string())?
            .as_ref()
            .map(|build_set| build_set.as_str().to_owned());
        Ok(resolve_project_native_plugin_admission(
            project_root,
            build_set_id.as_deref(),
            target,
            &project.manifest().plugins,
        ))
    }

    /// Resolves native authority without promoting project-controlled files to trusted code.
    /// Verified host policy or build-embedded expectations must be supplied independently before
    /// a project-native DLL can execute in the editor or its Play runtime.
    pub(crate) fn native_plugin_artifact_authority_for_project(
        &self,
        project_root: &Path,
    ) -> Result<NativePluginArtifactAuthority, String> {
        let project = self
            .host
            .current_project_snapshot_for_plugin_authority()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "native plugin authority requires an open project".to_string())?;
        if project.paths().root() != project_root {
            return Err(format!(
                "native plugin authority project root changed from {} to {}",
                project.paths().root().display(),
                project_root.display()
            ));
        }

        Ok(NativePluginArtifactAuthority::deny_all())
    }
}

#[cfg(test)]
#[path = "tests/authority.rs"]
mod tests;
