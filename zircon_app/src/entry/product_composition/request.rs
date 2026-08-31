use std::path::{Path, PathBuf};

use zircon_runtime::core::CoreError;
use zircon_runtime::plugin::native::host::NativePluginHostHandle;
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
    native_plugin_export_root: Option<PathBuf>,
}

impl ProductCompositionRequest {
    /// Starts a composition transaction from an unresolved product entry request.
    pub fn new(config: EntryConfig) -> Self {
        Self {
            config: ProductCompositionConfigRequest::Entry(config),
            registration_source: RuntimePluginRegistrationSource::FirstPartyCatalog,
            runtime_plugin_registrations: Vec::new(),
            runtime_plugin_feature_registrations: Vec::new(),
            native_plugin_export_root: None,
        }
    }

    pub(crate) fn from_resolved_config(config: ResolvedProductHostConfig) -> Self {
        Self {
            config: ProductCompositionConfigRequest::Resolved(config),
            registration_source: RuntimePluginRegistrationSource::FirstPartyCatalog,
            runtime_plugin_registrations: Vec::new(),
            runtime_plugin_feature_registrations: Vec::new(),
            native_plugin_export_root: None,
        }
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
    pub fn compose(self) -> Result<ProductComposition, CoreError> {
        self.prepare()?.compose()
    }

    fn prepare(self) -> Result<PreparedProductComposition, CoreError> {
        let resolved_config = match self.config {
            ProductCompositionConfigRequest::Entry(config) => resolve_product_host_config(&config)?,
            ProductCompositionConfigRequest::Resolved(config) => config,
        };
        let mut runtime_plugin_registrations = self.runtime_plugin_registrations;
        let mut runtime_plugin_feature_registrations = self.runtime_plugin_feature_registrations;
        let mut diagnostics = Vec::new();
        let native_plugin_host = match self.native_plugin_export_root {
            Some(export_root) => {
                let native_plugin_host = NativePluginHostHandle::default();
                let native_report = native_plugin_host
                    .load_runtime_plugins_from_export_root(export_root)
                    .map_err(|error| {
                        CoreError::Initialization("NativePluginHostHandle".to_owned(), error)
                    })?;
                diagnostics = native_report.diagnostics;
                runtime_plugin_registrations
                    .extend(native_report.runtime_plugin_registration_reports);
                runtime_plugin_feature_registrations
                    .extend(native_report.runtime_plugin_feature_registration_reports);
                Some(native_plugin_host)
            }
            None => None,
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

impl PreparedProductComposition {
    fn compose(self) -> Result<ProductComposition, CoreError> {
        let module_selection_report = self.entry.module_selection_report();
        let plugin_bridge_lifecycle_state =
            self.entry.runtime_plugin_bridge_lifecycle_state().cloned();
        let compiled_project_plugin_plan = self.entry.compiled_project_plugin_plan();
        let core = self.entry.bootstrap()?;
        Ok(ProductComposition::new(
            self.resolved_config,
            module_selection_report,
            self.diagnostics,
            core,
            plugin_bridge_lifecycle_state,
            compiled_project_plugin_plan,
            self.native_plugin_host,
        ))
    }
}
