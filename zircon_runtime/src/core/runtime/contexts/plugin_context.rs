use std::path::PathBuf;

use super::super::weak::CoreWeak;

#[derive(Clone, Debug)]
pub struct PluginContext {
    pub plugin_name: String,
    pub core: CoreWeak,
    pub package_root: Option<PathBuf>,
    pub source_root: Option<PathBuf>,
    pub data_root: Option<PathBuf>,
}
