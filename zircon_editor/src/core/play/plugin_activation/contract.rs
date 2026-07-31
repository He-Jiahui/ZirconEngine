use std::{path::Path, sync::Arc};

use super::PluginBridgeActivationReport;

pub trait PluginBridgeActivation: Send + Sync {
    fn activate(&self, project_root: Option<&Path>)
        -> Result<PluginBridgeActivationReport, String>;

    fn deactivate(&self) -> Result<PluginBridgeActivationReport, String>;
}

pub type SharedPluginBridgeActivation = Arc<dyn PluginBridgeActivation>;
