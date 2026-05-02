use crate::ui::template::UiCompiledDocument;
use zircon_runtime_interface::ui::template::UiInvalidationReport;

#[derive(Clone, Debug, PartialEq)]
pub struct UiCompileCacheOutcome {
    pub compiled: UiCompiledDocument,
    pub cache_hit: bool,
    pub invalidation_report: UiInvalidationReport,
}
