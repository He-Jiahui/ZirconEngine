use crate::asset::{
    NativeAssetImportCommandHost, NativeAssetImportCommandReport, NativeAssetImportCommandStatus,
};

use super::LoadedNativePlugin;

impl NativeAssetImportCommandHost for LoadedNativePlugin {
    fn command_host_id(&self) -> &str {
        &self.plugin_id
    }

    fn invoke_asset_import_command(
        &self,
        command: &str,
        payload: &[u8],
    ) -> NativeAssetImportCommandReport {
        let report = LoadedNativePlugin::invoke_runtime_command(self, command, payload);
        let status = match report.status_code {
            super::super::ZIRCON_NATIVE_PLUGIN_STATUS_OK => NativeAssetImportCommandStatus::Ok,
            super::super::ZIRCON_NATIVE_PLUGIN_STATUS_ERROR => {
                NativeAssetImportCommandStatus::Error
            }
            super::super::ZIRCON_NATIVE_PLUGIN_STATUS_DENIED => {
                NativeAssetImportCommandStatus::Denied
            }
            super::super::ZIRCON_NATIVE_PLUGIN_STATUS_PANIC => {
                NativeAssetImportCommandStatus::Panic
            }
            status => NativeAssetImportCommandStatus::Unknown(status),
        };
        NativeAssetImportCommandReport {
            status,
            diagnostics: report.diagnostics,
            payload: report.payload,
        }
    }
}
