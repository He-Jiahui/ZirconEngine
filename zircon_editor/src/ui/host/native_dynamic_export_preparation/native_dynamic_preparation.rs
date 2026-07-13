use std::path::PathBuf;

use super::super::editor_manager_plugins_export::EditorExportCargoInvocation;
use super::cleanup::cleanup_native_dynamic_preparation;

#[derive(Debug)]
pub(in crate::ui::host) struct NativeDynamicPreparation {
    pub(in crate::ui::host) plugin_root: PathBuf,
    pub(in crate::ui::host) build_root: PathBuf,
    pub(in crate::ui::host) cargo_invocations: Vec<EditorExportCargoInvocation>,
    pub(in crate::ui::host) diagnostics: Vec<String>,
}

impl Drop for NativeDynamicPreparation {
    fn drop(&mut self) {
        let _ = cleanup_native_dynamic_preparation(self);
    }
}
