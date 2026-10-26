//! Admission-filtered package metadata, retained only for the current project.

use std::path::{Path, PathBuf};

use crate::core::project::ProjectPreflightCompositionPlan;
use zircon_runtime::asset::project::ProjectScriptManifest;
use zircon_runtime::script::{discover_vm_plugin_packages_with_limits, VmPluginDiscoveryLimits};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScriptPackageChoice {
    pub(crate) package: String,
    pub(crate) module: String,
}

#[derive(Default)]
pub(super) struct ProjectScriptAuthoringCache {
    root: Option<PathBuf>,
    allowed: bool,
    scripts: ProjectScriptManifest,
    packages: Option<Result<Vec<ScriptPackageChoice>, String>>,
}

impl ProjectScriptAuthoringCache {
    pub(super) fn configure(&mut self, root: &Path, plan: &ProjectPreflightCompositionPlan) {
        *self = Self {
            root: Some(root.to_owned()),
            allowed: plan.allows_project_scripts(),
            scripts: plan.approved_project_scripts().clone(),
            packages: None,
        };
    }

    pub(super) fn allows(&self, root: &Path) -> bool {
        self.allowed && self.root.as_deref() == Some(root)
    }

    pub(super) fn packages(
        &mut self,
        root: &Path,
        refresh: bool,
    ) -> Result<Vec<ScriptPackageChoice>, String> {
        if !self.allows(root) {
            return Ok(Vec::new());
        }
        if refresh || self.packages.is_none() {
            self.packages = Some(discover_packages(root, &self.scripts));
        }
        self.packages
            .as_ref()
            .cloned()
            .unwrap_or_else(|| Ok(Vec::new()))
    }
}

fn discover_packages(
    root: &Path,
    scripts: &ProjectScriptManifest,
) -> Result<Vec<ScriptPackageChoice>, String> {
    let mut packages = Vec::new();
    for relative in &scripts.package_roots {
        let discovered = discover_vm_plugin_packages_with_limits(
            root.join(relative),
            VmPluginDiscoveryLimits {
                max_manifests: 128,
                max_entries: 4096,
                max_wall_time: std::time::Duration::from_secs(1),
                ..Default::default()
            },
        )
        .map_err(|error| error.to_string())?;
        for found in discovered {
            let manifest = found.package.manifest;
            let module = found
                .package
                .zr_vm_project
                .map(|source| source.entry_module)
                .unwrap_or(manifest.entry);
            let choice = ScriptPackageChoice {
                package: manifest.name,
                module,
            };
            if !packages.contains(&choice) {
                packages.push(choice);
            }
            if packages.len() > 256 {
                return Err("project script package catalog exceeds 256 packages".into());
            }
        }
    }
    packages.sort_by(|a, b| (&a.package, &a.module).cmp(&(&b.package, &b.module)));
    Ok(packages)
}

impl super::EditorManager {
    pub(super) fn clear_script_binding_catalog(&self) {
        *self
            .script_binding_catalog
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Default::default();
    }

    pub(super) fn current_script_binding_packages(
        &self,
        refresh: bool,
    ) -> Result<Vec<ScriptPackageChoice>, String> {
        let project = self
            .host
            .current_project_snapshot()
            .map_err(|e| e.to_string())?
            .ok_or("script authoring requires an active project")?;
        self.script_binding_packages(project.paths().root(), refresh)
    }

    pub(super) fn allows_current_script_binding_authoring(&self) -> bool {
        self.host
            .current_project_snapshot()
            .ok()
            .flatten()
            .is_some_and(|project| self.allows_script_binding_authoring(project.paths().root()))
    }

    pub(crate) fn allows_script_binding_authoring(&self, root: &Path) -> bool {
        self.script_binding_catalog
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .allows(root)
    }

    pub(crate) fn script_binding_packages(
        &self,
        root: &Path,
        refresh: bool,
    ) -> Result<Vec<ScriptPackageChoice>, String> {
        self.script_binding_catalog
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .packages(root, refresh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::project::ProjectPreflightCompositionProfile;
    use zircon_runtime::core::framework::project::ProjectPluginManifest;

    fn fixture_root() -> PathBuf {
        std::env::var_os("ZIRCON_G02_UI_ARTIFACT_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
    }

    fn plan(profile: ProjectPreflightCompositionProfile) -> ProjectPreflightCompositionPlan {
        ProjectPreflightCompositionPlan::compile(
            profile,
            &ProjectPluginManifest::default(),
            &ProjectScriptManifest {
                package_roots: vec!["scripts".into()],
                startup_packages: Vec::new(),
            },
        )
    }

    #[test]
    fn script_binding_package_cache_uses_approved_roots_and_never_scans_safe_or_other_projects() {
        let root = fixture_root().join(format!("zircon_g02_packages_{}", uuid::Uuid::new_v4()));
        let scripts = root.join("scripts/package");
        std::fs::create_dir_all(&scripts).unwrap();
        std::fs::write(
            scripts.join("plugin.toml"),
            "name = \"orb_run\"\nversion = \"0.1.0\"\nentry = \"main\"\nbackend = \"test\"\n",
        )
        .unwrap();
        let mut cache = ProjectScriptAuthoringCache::default();
        cache.configure(&root, &plan(ProjectPreflightCompositionProfile::Safe));
        assert!(!cache.allows(&root));
        assert!(cache.packages(&root, true).unwrap().is_empty());
        cache.configure(&root, &plan(ProjectPreflightCompositionProfile::Normal));
        assert_eq!(
            cache.packages(&root, true).unwrap(),
            vec![ScriptPackageChoice {
                package: "orb_run".into(),
                module: "main".into()
            }]
        );
        std::fs::remove_file(scripts.join("plugin.toml")).unwrap();
        assert_eq!(cache.packages(&root, false).unwrap().len(), 1);
        assert!(cache.packages(&root, true).unwrap().is_empty());
        assert!(cache
            .packages(&root.join("other"), false)
            .unwrap()
            .is_empty());
        cache.configure(&root, &plan(ProjectPreflightCompositionProfile::Recovery));
        assert!(cache.packages(&root, true).unwrap().is_empty());
        assert!(root.starts_with(fixture_root()));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn script_binding_discovery_failure_is_retained_for_visible_feedback() {
        let root = fixture_root().join(format!("zircon_g02_missing_{}", uuid::Uuid::new_v4()));
        let mut cache = ProjectScriptAuthoringCache::default();
        cache.configure(&root, &plan(ProjectPreflightCompositionProfile::Normal));
        assert!(cache.packages(&root, true).is_err());
        assert!(cache.packages(&root, false).is_err());
    }
}
