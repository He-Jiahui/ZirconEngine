use std::path::PathBuf;

use libloading::Library;

use super::{NativePluginDescriptor, NativePluginEntryReport};

pub struct LoadedNativePlugin {
    pub plugin_id: String,
    pub library_path: PathBuf,
    pub descriptor: Option<NativePluginDescriptor>,
    pub runtime_entry_report: Option<NativePluginEntryReport>,
    pub editor_entry_report: Option<NativePluginEntryReport>,
    pub(super) library: Library,
}

impl LoadedNativePlugin {
    pub fn is_loaded(&self) -> bool {
        let _ = &self.library;
        true
    }
}

impl std::fmt::Debug for LoadedNativePlugin {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LoadedNativePlugin")
            .field("plugin_id", &self.plugin_id)
            .field("library_path", &self.library_path)
            .field("descriptor", &self.descriptor)
            .field("runtime_entry_report", &self.runtime_entry_report)
            .field("editor_entry_report", &self.editor_entry_report)
            .finish_non_exhaustive()
    }
}
