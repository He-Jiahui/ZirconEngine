use std::path::{Path, PathBuf};

use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime::core::CoreError;
use zircon_runtime::plugin::native::NativePluginArtifactAuthority;
use zircon_runtime::{
    core::framework::project::ExportProfile, core::framework::project::ProjectPluginManifest,
    plugin::PluginModuleKind, plugin::PluginPackageRole,
    plugin::RuntimePluginFeatureRegistrationReport, plugin::RuntimePluginRegistrationReport,
};

use super::{EntryConfig, ProductComposition, ProductCompositionRequest, ProductRoleRequest};

/// Admitted export configuration and its linked runtime plugin reports.
#[derive(Clone, Debug)]
pub struct ExportRuntimeBootstrapConfig {
    native_plugin_artifact_authority: NativePluginArtifactAuthority,
    config_file_path: Option<PathBuf>,
    /// Plugin selections serialized into the export receipt.
    pub project_plugins: ProjectPluginManifest,
    /// Target, runtime profile, and packaging identity for the exported product.
    pub export_profile: ExportProfile,
    /// Linked runtime plugin registrations emitted by generated provider tables.
    pub runtime_plugin_registrations: Vec<RuntimePluginRegistrationReport>,
    /// Linked runtime feature registrations emitted by generated provider tables.
    pub runtime_plugin_feature_registrations: Vec<RuntimePluginFeatureRegistrationReport>,
}

/// Deferred linked runtime plugin registration provider used by generated exports.
#[derive(Clone, Copy, Debug)]
pub struct ExportRuntimePluginRegistrationProvider {
    register: fn() -> RuntimePluginRegistrationReport,
}

impl ExportRuntimePluginRegistrationProvider {
    /// Stores a registration function without executing it in generated code.
    pub const fn new(register: fn() -> RuntimePluginRegistrationReport) -> Self {
        Self { register }
    }

    fn into_report(self) -> RuntimePluginRegistrationReport {
        (self.register)()
    }
}

/// Deferred linked runtime feature registration provider used by generated exports.
#[derive(Clone, Copy, Debug)]
pub struct ExportRuntimePluginFeatureRegistrationProvider {
    register: fn() -> RuntimePluginFeatureRegistrationReport,
    provider_package_id: Option<&'static str>,
    admitted_source_identity: Option<AdmittedLinkedFeatureIdentity>,
}

#[derive(Clone, Copy, Debug)]
struct AdmittedLinkedFeatureIdentity {
    feature_id: &'static str,
    owner_plugin_id: &'static str,
    provider_package_id: &'static str,
    runtime_crate: &'static str,
    package_role: PluginPackageRole,
}

impl ExportRuntimePluginFeatureRegistrationProvider {
    /// Stores a feature registration function without executing it in generated code.
    pub const fn new(register: fn() -> RuntimePluginFeatureRegistrationReport) -> Self {
        Self {
            register,
            provider_package_id: None,
            admitted_source_identity: None,
        }
    }

    /// Overrides the package identity attached to the generated feature report.
    pub const fn with_provider_package_id(mut self, provider_package_id: &'static str) -> Self {
        self.provider_package_id = Some(provider_package_id);
        self
    }

    /// Binds the admitted package role to the feature identity emitted by the linked crate.
    pub const fn with_admitted_source_identity(
        mut self,
        feature_id: &'static str,
        owner_plugin_id: &'static str,
        provider_package_id: &'static str,
        runtime_crate: &'static str,
        package_role: PluginPackageRole,
    ) -> Self {
        self.admitted_source_identity = Some(AdmittedLinkedFeatureIdentity {
            feature_id,
            owner_plugin_id,
            provider_package_id,
            runtime_crate,
            package_role,
        });
        self
    }

    fn into_report(self) -> RuntimePluginFeatureRegistrationReport {
        let mut report = (self.register)();
        let role = match self.admitted_source_identity {
            Some(identity)
                if identity.matches(&report)
                    && self
                        .provider_package_id
                        .is_none_or(|id| id == identity.provider_package_id) =>
            {
                identity.package_role
            }
            Some(identity) => {
                report.diagnostics.push(format!(
                    "linked feature registration {} does not match admitted source identity {} owned by {} from {}",
                    report.manifest.id,
                    identity.feature_id,
                    identity.owner_plugin_id,
                    identity.runtime_crate
                ));
                PluginPackageRole::TestFixture
            }
            None => PluginPackageRole::TestFixture,
        };
        let provider_package_id = self
            .admitted_source_identity
            .map(|identity| identity.provider_package_id)
            .or(self.provider_package_id);
        let report = match provider_package_id {
            Some(provider_package_id) => report.with_provider_package_id(provider_package_id),
            None => report,
        };
        report.with_provider_package_role(role)
    }
}

impl AdmittedLinkedFeatureIdentity {
    fn matches(self, report: &RuntimePluginFeatureRegistrationReport) -> bool {
        if report.manifest.id != self.feature_id
            || report.manifest.owner_plugin_id != self.owner_plugin_id
            || report
                .provider_package_id
                .as_deref()
                .is_some_and(|provider| provider != self.provider_package_id)
            || report
                .manifest
                .provider_package_id
                .as_deref()
                .is_some_and(|provider| provider != self.provider_package_id)
            || report
                .project_selection
                .provider_package_id
                .as_deref()
                .is_some_and(|provider| provider != self.provider_package_id)
        {
            return false;
        }
        let mut runtime_modules = report
            .manifest
            .modules
            .iter()
            .filter(|module| module.kind == PluginModuleKind::Runtime);
        runtime_modules
            .next()
            .is_some_and(|module| module.crate_name == self.runtime_crate)
            && runtime_modules.all(|module| module.crate_name == self.runtime_crate)
    }
}

impl ExportRuntimeBootstrapConfig {
    /// Creates an export request from its single profile and plugin-manifest authority.
    pub fn new(project_plugins: ProjectPluginManifest, export_profile: ExportProfile) -> Self {
        Self {
            native_plugin_artifact_authority: NativePluginArtifactAuthority::deny_all(),
            config_file_path: None,
            project_plugins,
            export_profile,
            runtime_plugin_registrations: Vec::new(),
            runtime_plugin_feature_registrations: Vec::new(),
        }
    }

    /// Sets the host's authority for admitting native plugin artifacts.
    pub fn with_native_plugin_artifact_authority(
        mut self,
        authority: NativePluginArtifactAuthority,
    ) -> Self {
        self.native_plugin_artifact_authority = authority;
        self
    }

    /// Selects a host-owned Foundation config file for this composition.
    pub fn with_config_file_path(mut self, path: PathBuf) -> Self {
        self.config_file_path = Some(path);
        self
    }

    /// Appends already materialized linked runtime plugin reports.
    pub fn with_runtime_plugin_registrations(
        mut self,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Self {
        self.runtime_plugin_registrations.extend(registrations);
        self
    }

    /// Executes deferred linked runtime plugin providers at the handwritten boundary.
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

    /// Appends already materialized linked runtime feature reports.
    pub fn with_runtime_plugin_feature_registrations(
        mut self,
        registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Self {
        self.runtime_plugin_feature_registrations
            .extend(registrations);
        self
    }

    /// Executes deferred linked runtime feature providers at the handwritten boundary.
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

    /// Projects the export receipt into an unresolved product entry request.
    pub fn entry_config(&self) -> EntryConfig {
        EntryConfig::for_product_role(ProductRoleRequest::from_export_profile(
            &self.export_profile,
        ))
        .with_export_project_plugins(self.project_plugins.clone())
        .with_export_profile(self.export_profile.clone())
    }

    fn into_parts(
        self,
    ) -> (
        EntryConfig,
        Vec<RuntimePluginRegistrationReport>,
        Vec<RuntimePluginFeatureRegistrationReport>,
        Option<PathBuf>,
    ) {
        let product_role = ProductRoleRequest::from_export_profile(&self.export_profile);
        (
            EntryConfig::for_product_role(product_role)
                .with_export_project_plugins(self.project_plugins)
                .with_export_profile(self.export_profile),
            self.runtime_plugin_registrations,
            self.runtime_plugin_feature_registrations,
            self.config_file_path,
        )
    }
}

/// Composes a linked/static exported runtime and retains its complete owner set.
pub fn bootstrap_export_runtime(
    config: ExportRuntimeBootstrapConfig,
) -> Result<ProductComposition, crate::entry::ProductCompositionFailure> {
    let (entry_config, registrations, feature_registrations, config_file_path) =
        config.into_parts();
    let mut request = ProductCompositionRequest::new(entry_config)
        .with_runtime_plugin_and_feature_registrations(registrations, feature_registrations);
    if let Some(path) = config_file_path {
        request = request.with_config_file_path(path);
    }
    request.compose()
}

/// Composes an exported runtime with linked and native dynamic plugin reports.
pub fn bootstrap_export_runtime_with_native_plugins_from_export_root(
    config: ExportRuntimeBootstrapConfig,
    export_root: impl AsRef<Path>,
) -> Result<ProductComposition, crate::entry::ProductCompositionFailure> {
    let authority = config.native_plugin_artifact_authority.clone();
    let (entry_config, registrations, feature_registrations, config_file_path) =
        config.into_parts();
    let mut request = ProductCompositionRequest::new(entry_config)
        .with_runtime_plugin_and_feature_registrations(registrations, feature_registrations)
        .with_native_plugins_from_export_root(export_root)
        .with_native_plugin_artifact_authority(authority);
    if let Some(path) = config_file_path {
        request = request.with_config_file_path(path);
    }
    request.compose()
}

/// Resolves the nearest export root visible from the executable or working directory.
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
#[path = "tests/export_bootstrap_unit.rs"]
mod tests;
