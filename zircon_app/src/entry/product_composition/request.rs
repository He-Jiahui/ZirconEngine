use std::path::{Path, PathBuf};

use crate::entry::product_shutdown::retained_owner::{
    ensure_product_admission, ProductCompositionFailure,
};
use std::time::{Duration, Instant};
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::core::{CoreError, CoreRuntime};
use zircon_runtime::foundation;
use zircon_runtime::plugin::native::host::NativePluginHostHandle;
use zircon_runtime::plugin::native::{NativePluginArtifactAuthority, NativePluginBehaviorHealth};
use zircon_runtime::plugin::{
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

use crate::entry::engine_entry::resolve_product_host_config;

use super::super::{
    BuiltinEngineEntry, EngineEntry, EntryConfig, EntryModuleSelectionReport,
    ResolvedProductHostConfig,
};
use super::ProductComposition;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuntimePluginRegistrationSource {
    FirstPartyCatalog,
    ExplicitReports,
}

#[derive(Clone, Debug)]
enum ProductCompositionConfigRequest {
    Entry(EntryConfig),
    Resolved(ResolvedProductHostConfig),
}

/// Single request surface for product config resolution, provider selection, and Core bootstrap.
#[must_use = "a product composition request must be inspected or composed"]
#[derive(Clone, Debug)]
pub struct ProductCompositionRequest {
    config: ProductCompositionConfigRequest,
    registration_source: RuntimePluginRegistrationSource,
    runtime_plugin_registrations: Vec<RuntimePluginRegistrationReport>,
    runtime_plugin_feature_registrations: Vec<RuntimePluginFeatureRegistrationReport>,
    config_file_path: Option<PathBuf>,
    native_plugin_export_root: Option<PathBuf>,
    native_plugin_artifact_authority: NativePluginArtifactAuthority,
}

impl ProductCompositionRequest {
    /// Starts a composition transaction from an unresolved product entry request.
    pub fn new(config: EntryConfig) -> Self {
        Self {
            config: ProductCompositionConfigRequest::Entry(config),
            registration_source: RuntimePluginRegistrationSource::FirstPartyCatalog,
            runtime_plugin_registrations: Vec::new(),
            runtime_plugin_feature_registrations: Vec::new(),
            config_file_path: None,
            native_plugin_export_root: None,
            native_plugin_artifact_authority: NativePluginArtifactAuthority::deny_all(),
        }
    }

    pub(crate) fn from_resolved_config(config: ResolvedProductHostConfig) -> Self {
        Self {
            config: ProductCompositionConfigRequest::Resolved(config),
            registration_source: RuntimePluginRegistrationSource::FirstPartyCatalog,
            runtime_plugin_registrations: Vec::new(),
            runtime_plugin_feature_registrations: Vec::new(),
            config_file_path: None,
            native_plugin_export_root: None,
            native_plugin_artifact_authority: NativePluginArtifactAuthority::deny_all(),
        }
    }

    /// Binds one absolute host-owned Foundation persistence file for this composition.
    /// The path is checked against the canonical Foundation descriptor during preparation.
    pub fn with_config_file_path(mut self, path: PathBuf) -> Self {
        self.config_file_path = Some(path);
        self
    }

    /// Replaces catalog discovery with explicit runtime plugin registration reports.
    pub fn with_runtime_plugin_registrations(
        mut self,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Self {
        self.registration_source = RuntimePluginRegistrationSource::ExplicitReports;
        self.runtime_plugin_registrations.extend(registrations);
        self
    }

    /// Replaces catalog discovery with explicit runtime plugin feature reports.
    pub fn with_runtime_plugin_feature_registrations(
        mut self,
        registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Self {
        self.registration_source = RuntimePluginRegistrationSource::ExplicitReports;
        self.runtime_plugin_feature_registrations
            .extend(registrations);
        self
    }

    /// Replaces catalog discovery with explicit plugin and feature registration reports.
    pub fn with_runtime_plugin_and_feature_registrations(
        self,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Self {
        self.with_runtime_plugin_registrations(registrations)
            .with_runtime_plugin_feature_registrations(feature_registrations)
    }

    /// Sets the host's authority for admitting native plugin artifacts.
    pub fn with_native_plugin_artifact_authority(
        mut self,
        authority: NativePluginArtifactAuthority,
    ) -> Self {
        self.native_plugin_artifact_authority = authority;
        self
    }

    /// Adds native plugin packages discovered below an admitted export root.
    pub fn with_native_plugins_from_export_root(mut self, export_root: impl AsRef<Path>) -> Self {
        self.registration_source = RuntimePluginRegistrationSource::ExplicitReports;
        self.native_plugin_export_root = Some(export_root.as_ref().to_path_buf());
        self
    }

    /// Prepares the transaction and returns its immutable module selection receipt.
    pub fn module_selection_report(self) -> Result<EntryModuleSelectionReport, CoreError> {
        Ok(self.prepare()?.entry.module_selection_report())
    }

    /// Formats diagnostics from the same preparation path used by full composition.
    pub fn module_selection_diagnostics(self) -> Result<String, CoreError> {
        Ok(self.module_selection_report()?.format_diagnostics())
    }

    /// Resolves, compiles, and bootstraps one complete product generation.
    pub fn compose(self) -> Result<ProductComposition, ProductCompositionFailure> {
        ensure_product_admission()?;
        self.prepare()?.compose()
    }

    fn prepare(self) -> Result<PreparedProductComposition, CoreError> {
        let resolved_config = match self.config {
            ProductCompositionConfigRequest::Entry(config) => resolve_product_host_config(&config)?,
            ProductCompositionConfigRequest::Resolved(config) => config,
        };
        // Reject invalid host storage authority before native loading can have effects.
        // The selected entry still binds the path to its actual Foundation descriptor below.
        if let Some(path) = self.config_file_path.as_ref() {
            let mut descriptor = foundation::module_descriptor();
            foundation::bind_config_file_path(&mut descriptor, path.clone())?;
        }
        let mut runtime_plugin_registrations = self.runtime_plugin_registrations;
        let mut runtime_plugin_feature_registrations = self.runtime_plugin_feature_registrations;
        let selected_native_dynamic_ids = resolved_config
            .project_plugin_manifest()
            .map(|manifest| {
                manifest
                    .enabled_for_target(resolved_config.target_mode())
                    .filter(|selection| {
                        selection.packaging == ExportPackagingStrategy::NativeDynamic
                    })
                    .map(|selection| selection.id.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        // Caller supplied discovery reports cannot attest to a live native entry.
        // Only the host's admitted reports below may describe NativeDynamic providers.
        runtime_plugin_registrations.retain(|report| {
            report.project_selection.packaging != ExportPackagingStrategy::NativeDynamic
                && !selected_native_dynamic_ids.contains(&report.package_manifest.id)
        });
        runtime_plugin_feature_registrations.retain(|report| {
            let provider_id = report
                .provider_package_id
                .as_deref()
                .unwrap_or(&report.manifest.owner_plugin_id);
            report.project_selection.packaging != ExportPackagingStrategy::NativeDynamic
                && !selected_native_dynamic_ids
                    .iter()
                    .any(|id| id == provider_id)
        });
        let mut diagnostics = Vec::new();
        let native_plugin_host = match self.native_plugin_export_root {
            Some(export_root) => {
                self.native_plugin_artifact_authority
                    .validate_runtime_target(resolved_config.target_mode())
                    .map_err(|error| {
                        CoreError::Initialization(
                            "NativePluginAdmission".to_string(),
                            error.to_string(),
                        )
                    })?;
                let native_plugin_host = NativePluginHostHandle::with_artifact_authority(
                    self.native_plugin_artifact_authority,
                );
                let native_report = native_plugin_host
                    .load_runtime_plugins_from_export_root(export_root)
                    .map_err(|error| {
                        CoreError::Initialization("NativePluginHostHandle".to_owned(), error)
                    })?;
                let admitted_plugin_ids = native_report
                    .loaded_plugin_ids
                    .iter()
                    .filter(|id| {
                        native_plugin_host
                            .runtime_behavior_descriptor(id.as_str())
                            .ok()
                            .and_then(|descriptor| descriptor.validation_report)
                            .is_some_and(|report| {
                                report.health != NativePluginBehaviorHealth::Invalid
                            })
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                diagnostics = native_report.diagnostics;
                if let Some(manifest) = resolved_config.project_plugin_manifest() {
                    for selection in manifest.enabled_for_target(resolved_config.target_mode()) {
                        if selection.required
                            && selection.packaging == ExportPackagingStrategy::NativeDynamic
                        {
                            let has_registration = native_report
                                .runtime_plugin_registration_reports
                                .iter()
                                .any(|report| report.package_manifest.id == selection.id);
                            if !(has_registration && admitted_plugin_ids.contains(&selection.id)) {
                                return Err(CoreError::Initialization(
                                    "NativePluginAdmission".to_string(),
                                    format!(
                                        "required native plugin {} was not admitted: {}",
                                        selection.id,
                                        diagnostics.join("; ")
                                    ),
                                ));
                            }
                        }
                    }
                }
                runtime_plugin_registrations.extend(
                    native_report
                        .runtime_plugin_registration_reports
                        .into_iter()
                        .filter(|report| admitted_plugin_ids.contains(&report.package_manifest.id)),
                );
                runtime_plugin_feature_registrations.extend(
                    native_report
                        .runtime_plugin_feature_registration_reports
                        .into_iter()
                        .filter(|report| {
                            let provider_id = report
                                .provider_package_id
                                .as_deref()
                                .unwrap_or(&report.manifest.owner_plugin_id);
                            admitted_plugin_ids.iter().any(|id| id == provider_id)
                        }),
                );
                Some(native_plugin_host)
            }
            None => {
                if let Some(manifest) = resolved_config.project_plugin_manifest() {
                    if let Some(selection) = manifest
                        .enabled_for_target(resolved_config.target_mode())
                        .find(|selection| {
                            selection.required
                                && selection.packaging == ExportPackagingStrategy::NativeDynamic
                        })
                    {
                        return Err(CoreError::Initialization(
                            "NativePluginAdmission".to_string(),
                            format!(
                                "required native plugin {} was not admitted: no native export root",
                                selection.id
                            ),
                        ));
                    }
                }
                None
            }
        };

        let entry = match self.registration_source {
            RuntimePluginRegistrationSource::FirstPartyCatalog =>
                BuiltinEngineEntry::for_resolved_config_with_first_party_runtime_plugin_registrations(
                    &resolved_config,
                )?,
            RuntimePluginRegistrationSource::ExplicitReports => {
                BuiltinEngineEntry::for_resolved_config_with_runtime_plugin_and_feature_registrations(
                    &resolved_config,
                    runtime_plugin_registrations,
                    runtime_plugin_feature_registrations,
                )?
            }
        };

        let entry = match self.config_file_path {
            Some(path) => entry.with_config_file_path(path)?,
            None => entry,
        };

        Ok(PreparedProductComposition {
            resolved_config,
            entry,
            native_plugin_host,
            diagnostics,
        })
    }
}

struct PreparedProductComposition {
    resolved_config: ResolvedProductHostConfig,
    entry: BuiltinEngineEntry,
    native_plugin_host: Option<NativePluginHostHandle>,
    diagnostics: Vec<String>,
}

#[cfg(test)]
#[path = "tests/request_native_admission_tests.rs"]
mod native_admission_tests;

impl PreparedProductComposition {
    fn compose(self) -> Result<ProductComposition, ProductCompositionFailure> {
        let module_selection_report = self.entry.module_selection_report();
        let plugin_bridge_lifecycle_state =
            self.entry.runtime_plugin_bridge_lifecycle_state().cloned();
        let compiled_project_plugin_plan = self.entry.compiled_project_plugin_plan();
        let runtime =
            CoreRuntime::try_new().map_err(ProductCompositionFailure::before_ownership)?;
        let composition = ProductComposition::new(
            self.resolved_config,
            module_selection_report,
            self.diagnostics,
            runtime,
            plugin_bridge_lifecycle_state,
            compiled_project_plugin_plan,
            self.native_plugin_host,
        );
        if let Err(primary) = self.entry.bootstrap(composition.runtime()) {
            return Err(composition.fail_until(primary, Instant::now() + Duration::from_secs(5)));
        }
        Ok(composition)
    }
}
