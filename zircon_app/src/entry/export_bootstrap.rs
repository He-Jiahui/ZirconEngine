use std::path::{Path, PathBuf};

use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime::core::{CoreError, CoreHandle};
use zircon_runtime::{
    core::framework::platform::RuntimeTargetMode, core::framework::project::ExportProfile,
    core::framework::project::ProjectPluginManifest,
    plugin::RuntimePluginFeatureRegistrationReport, plugin::RuntimePluginRegistrationReport,
};

use super::{
    EntryConfig, EntryProfile, EntryRunner, EntryRuntimeBootstrap, NativePluginRuntimeBootstrap,
};

#[derive(Clone, Debug)]
pub struct ExportRuntimeBootstrapConfig {
    pub entry_profile: EntryProfile,
    pub target_mode: RuntimeTargetMode,
    pub project_plugins: ProjectPluginManifest,
    pub export_profile: ExportProfile,
    pub runtime_plugin_registrations: Vec<RuntimePluginRegistrationReport>,
    pub runtime_plugin_feature_registrations: Vec<RuntimePluginFeatureRegistrationReport>,
}

#[derive(Clone, Copy, Debug)]
pub struct ExportRuntimePluginRegistrationProvider {
    register: fn() -> RuntimePluginRegistrationReport,
}

impl ExportRuntimePluginRegistrationProvider {
    pub const fn new(register: fn() -> RuntimePluginRegistrationReport) -> Self {
        Self { register }
    }

    fn into_report(self) -> RuntimePluginRegistrationReport {
        (self.register)()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ExportRuntimePluginFeatureRegistrationProvider {
    register: fn() -> RuntimePluginFeatureRegistrationReport,
    provider_package_id: Option<&'static str>,
}

impl ExportRuntimePluginFeatureRegistrationProvider {
    pub const fn new(register: fn() -> RuntimePluginFeatureRegistrationReport) -> Self {
        Self {
            register,
            provider_package_id: None,
        }
    }

    pub const fn with_provider_package_id(mut self, provider_package_id: &'static str) -> Self {
        self.provider_package_id = Some(provider_package_id);
        self
    }

    fn into_report(self) -> RuntimePluginFeatureRegistrationReport {
        let report = (self.register)();
        match self.provider_package_id {
            Some(provider_package_id) => report.with_provider_package_id(provider_package_id),
            None => report,
        }
    }
}

impl ExportRuntimeBootstrapConfig {
    pub fn new(
        entry_profile: EntryProfile,
        target_mode: RuntimeTargetMode,
        project_plugins: ProjectPluginManifest,
        export_profile: ExportProfile,
    ) -> Self {
        Self {
            entry_profile,
            target_mode,
            project_plugins,
            export_profile,
            runtime_plugin_registrations: Vec::new(),
            runtime_plugin_feature_registrations: Vec::new(),
        }
    }

    pub fn with_runtime_plugin_registrations(
        mut self,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Self {
        self.runtime_plugin_registrations.extend(registrations);
        self
    }

    pub fn with_runtime_plugin_registration_providers(
        mut self,
        providers: impl IntoIterator<Item = ExportRuntimePluginRegistrationProvider>,
    ) -> Self {
        self.runtime_plugin_registrations.extend(
            providers
                .into_iter()
                .map(ExportRuntimePluginRegistrationProvider::into_report),
        );
        self
    }

    pub fn with_runtime_plugin_feature_registrations(
        mut self,
        registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Self {
        self.runtime_plugin_feature_registrations
            .extend(registrations);
        self
    }

    pub fn with_runtime_plugin_feature_registration_providers(
        mut self,
        providers: impl IntoIterator<Item = ExportRuntimePluginFeatureRegistrationProvider>,
    ) -> Self {
        self.runtime_plugin_feature_registrations.extend(
            providers
                .into_iter()
                .map(ExportRuntimePluginFeatureRegistrationProvider::into_report),
        );
        self
    }

    pub fn entry_config(&self) -> EntryConfig {
        EntryConfig::new(self.entry_profile)
            .with_target_mode(self.target_mode)
            .with_project_plugins(self.project_plugins.clone())
            .with_export_profile(self.export_profile.clone())
    }

    fn into_parts(
        self,
    ) -> (
        EntryConfig,
        Vec<RuntimePluginRegistrationReport>,
        Vec<RuntimePluginFeatureRegistrationReport>,
    ) {
        (
            EntryConfig::new(self.entry_profile)
                .with_target_mode(self.target_mode)
                .with_project_plugins(self.project_plugins)
                .with_export_profile(self.export_profile),
            self.runtime_plugin_registrations,
            self.runtime_plugin_feature_registrations,
        )
    }
}

pub fn bootstrap_export_runtime(
    config: ExportRuntimeBootstrapConfig,
) -> Result<CoreHandle, CoreError> {
    Ok(bootstrap_export_runtime_with_report(config)?.into_core())
}

pub fn bootstrap_export_runtime_with_report(
    config: ExportRuntimeBootstrapConfig,
) -> Result<EntryRuntimeBootstrap, CoreError> {
    let (entry_config, registrations, feature_registrations) = config.into_parts();
    EntryRunner::bootstrap_with_runtime_plugin_and_feature_registrations_and_report(
        entry_config,
        registrations,
        feature_registrations,
    )
}

pub fn bootstrap_export_runtime_with_native_plugins_from_export_root(
    config: ExportRuntimeBootstrapConfig,
    export_root: impl AsRef<Path>,
) -> Result<NativePluginRuntimeBootstrap, CoreError> {
    let (entry_config, registrations, feature_registrations) = config.into_parts();
    EntryRunner::bootstrap_with_runtime_plugin_and_feature_registrations_and_native_plugins_from_export_root(
        entry_config,
        registrations,
        feature_registrations,
        export_root,
    )
}

pub fn discover_export_root() -> std::io::Result<PathBuf> {
    let current_exe = std::env::current_exe()?;
    let current_dir = std::env::current_dir()?;
    discover_export_root_from_paths(&current_exe, &current_dir)
}

fn discover_export_root_from_paths(
    current_exe: &Path,
    current_dir: &Path,
) -> std::io::Result<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(parent) = current_exe.parent() {
        candidates.extend(parent.ancestors().map(PathBuf::from));
    }
    candidates.extend(current_dir.ancestors().map(PathBuf::from));
    for candidate in candidates {
        let Ok(root) = ProjectPaths::resolve_existing(candidate) else {
            continue;
        };
        let Ok(manifest) = ProjectPaths::resolve_path_from(&root, "plugins/native_plugins.toml")
        else {
            continue;
        };
        if manifest.operation_path().exists() {
            return Ok(root.into_operation_path());
        }
    }
    ProjectPaths::resolve_existing(current_dir).map(|root| root.into_operation_path())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime::asset::project::ProjectPaths;

    use super::discover_export_root_from_paths;

    #[cfg(any(unix, windows))]
    #[test]
    fn export_root_discovery_keeps_the_physical_identity_of_a_product_alias() {
        let parent = unique_export_root("alias");
        let physical_root = parent.join("physical-export");
        fs::create_dir_all(physical_root.join("plugins")).unwrap();
        fs::create_dir_all(physical_root.join("bin")).unwrap();
        fs::write(physical_root.join("plugins/native_plugins.toml"), []).unwrap();
        let alias = parent.join("export-alias");
        create_directory_link(&physical_root, &alias);
        let working_directory = parent.join("working-directory");
        fs::create_dir_all(&working_directory).unwrap();

        let actual = discover_export_root_from_paths(
            &alias.join("bin/exported-product"),
            &working_directory,
        )
        .expect("export root discovery should resolve the product alias");
        let expected = ProjectPaths::resolve_existing_path(&physical_root).unwrap();

        fs::remove_dir_all(&parent).unwrap();
        assert_eq!(actual, expected);
    }

    fn unique_export_root(case_name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "zircon-export-bootstrap-{case_name}-{}-{timestamp}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        path
    }

    #[cfg(unix)]
    fn create_directory_link(target: &Path, link: &Path) {
        std::os::unix::fs::symlink(target, link).expect("create export-root alias fixture");
    }

    #[cfg(windows)]
    fn create_directory_link(target: &Path, link: &Path) {
        let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
        let output = std::process::Command::new("cmd")
            .args(["/D", "/S", "/C"])
            .arg(command)
            .output()
            .expect("start mklink for export-root alias fixture");
        assert!(
            output.status.success(),
            "create export-root junction fixture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
