use crate::asset::{
    NativeAssetImportCommandHost, NativeAssetImportCommandReport, NativeAssetImportCommandStatus,
};

use super::LoadedNativePlugin;

// 资产导入器只认识命令宿主和统一结果；此适配层把已加载插件的运行时命令接入导入管线，
// 并保留原始状态码与诊断，供导入任务决定失败、拒绝或未知 ABI 状态。
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
