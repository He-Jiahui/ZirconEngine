use zircon_runtime::plugin::BridgeDiagnosticsMatrix;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PluginBridgeActivationReport {
    pub diagnostics: Vec<String>,
    pub bridge_diagnostics: Option<BridgeDiagnosticsMatrix>,
}

impl PluginBridgeActivationReport {
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
