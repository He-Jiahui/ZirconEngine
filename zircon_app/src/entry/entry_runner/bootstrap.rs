use std::path::Path;

use zircon_runtime::core::{CoreError, CoreHandle};
use zircon_runtime::{NativePluginLoader, RuntimePluginRegistrationReport};

use crate::entry::{BuiltinEngineEntry, EngineEntry, EntryConfig};

use super::EntryRunner;

impl EntryRunner {
    pub fn bootstrap(config: EntryConfig) -> Result<CoreHandle, CoreError> {
        BuiltinEngineEntry::for_config(&config)?.bootstrap()
    }

    pub fn bootstrap_with_runtime_plugin_registrations(
        config: EntryConfig,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
    ) -> Result<CoreHandle, CoreError> {
        BuiltinEngineEntry::for_config_with_runtime_plugin_registrations(&config, registrations)?
            .bootstrap()
    }

    pub fn bootstrap_with_native_plugins_from_export_root(
        config: EntryConfig,
        export_root: impl AsRef<Path>,
    ) -> Result<CoreHandle, CoreError> {
        let native_report = NativePluginLoader.load_runtime_from_load_manifest(export_root);
        for diagnostic in &native_report.diagnostics {
            eprintln!("[zircon_app] native plugin warning: {diagnostic}");
        }
        for diagnostic in native_report.descriptor_diagnostics() {
            eprintln!("[zircon_app] native plugin descriptor warning: {diagnostic}");
        }
        for diagnostic in native_report.entry_diagnostics() {
            eprintln!("[zircon_app] native plugin entry warning: {diagnostic}");
        }
        BuiltinEngineEntry::for_config_with_runtime_plugin_registrations(
            &config,
            native_report.runtime_plugin_registration_reports(),
        )?
        .bootstrap()
    }
}
