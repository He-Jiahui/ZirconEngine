use super::RuntimeExtensionCatalogReport;

// 报告是否成功由 fatal_diagnostics 决定；普通 diagnostics 仍可保留可展示但不阻断的提示。
impl RuntimeExtensionCatalogReport {
    pub fn is_success(&self) -> bool {
        self.fatal_diagnostics.is_empty()
    }

    pub fn has_fatal_diagnostics(&self) -> bool {
        !self.fatal_diagnostics.is_empty()
    }
}
