use zircon_runtime::core::CoreError;
use zircon_runtime::plugin::RuntimePluginRegistrationReport;

use crate::entry::{
    EntryConfig, EntryModuleSelectionReport, ProductComposition, ProductCompositionRequest,
    ResolvedProductHostConfig,
};

use super::EntryRunner;

impl EntryRunner {
    /// Builds one complete product composition from an entry request.
    /// 返回值持有 Core 和插件生命周期；调用方须在宿主执行期间保留它，并安排其与动态会话的释放顺序。
    pub fn compose(
        config: EntryConfig,
    ) -> Result<ProductComposition, crate::entry::ProductCompositionFailure> {
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

    /// 编辑器在预检本地插件后沿相同组合路径装配，避免报告与实际注册使用不同配置。
    pub(crate) fn compose_resolved_with_runtime_plugin_registrations(
        config: ResolvedProductHostConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Result<ProductComposition, crate::entry::ProductCompositionFailure> {
        ProductCompositionRequest::from_resolved_config(config)
            .with_runtime_plugin_registrations(registrations)
            .compose()
    }
}
