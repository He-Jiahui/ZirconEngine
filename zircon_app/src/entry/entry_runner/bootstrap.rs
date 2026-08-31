use zircon_runtime::core::CoreError;
use zircon_runtime::plugin::RuntimePluginRegistrationReport;

use crate::entry::{
    EntryConfig, EntryModuleSelectionReport, ProductComposition, ProductCompositionRequest,
    ResolvedProductHostConfig,
};

use super::EntryRunner;

impl EntryRunner {
    /// Builds one complete product composition from an entry request.
    pub fn compose(config: EntryConfig) -> Result<ProductComposition, CoreError> {
        ProductCompositionRequest::new(config).compose()
    }

    /// Prepares a product request and returns its module selection receipt.
    pub fn module_selection_report(
        config: EntryConfig,
    ) -> Result<EntryModuleSelectionReport, CoreError> {
        ProductCompositionRequest::new(config).module_selection_report()
    }

    /// Formats diagnostics from the same preparation path used by composition.
    pub fn module_selection_diagnostics(config: EntryConfig) -> Result<String, CoreError> {
        ProductCompositionRequest::new(config).module_selection_diagnostics()
    }

    pub(crate) fn compose_resolved_with_runtime_plugin_registrations(
        config: ResolvedProductHostConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Result<ProductComposition, CoreError> {
        ProductCompositionRequest::from_resolved_config(config)
            .with_runtime_plugin_registrations(registrations)
            .compose()
    }
}
