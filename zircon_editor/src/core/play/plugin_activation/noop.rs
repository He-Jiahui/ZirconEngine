use std::path::Path;

use super::{PluginBridgeActivation, PluginBridgeActivationReport};

#[derive(Default)]
pub struct NoopPluginBridgeActivation;

impl PluginBridgeActivation for NoopPluginBridgeActivation {
    fn activate(
        &self,
        _project_root: Option<&Path>,
    ) -> Result<PluginBridgeActivationReport, String> {
        Ok(PluginBridgeActivationReport::default())
    }

    fn deactivate(&self) -> Result<PluginBridgeActivationReport, String> {
        Ok(PluginBridgeActivationReport::default())
    }
}
