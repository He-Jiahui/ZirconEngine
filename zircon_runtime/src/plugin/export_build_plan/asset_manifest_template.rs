//! SourceTemplate 将项目资源入口固化到导出资产目录；generated_files_for_profile 生成后由 materialize/generated.rs 写入，宿主按该清单启动。
use crate::asset::project::ProjectManifest;

/// SourceTemplate 的资产清单来源；当前失败路径仍返回文本，调用方不会收到序列化错误。
// TODO: [CR-PLUGIN-EXPORT-AUDIT-0005] 确认已准入 ProjectManifest 的 TOML 序列化是否绝不失败；否则注释文本会被当作有效资产清单写出。下一步核查字段序列化和错误样例。
pub(super) fn asset_manifest_template(manifest: &ProjectManifest) -> String {
    toml::to_string_pretty(manifest)
        .unwrap_or_else(|error| format!("# failed to serialize zircon project manifest: {error}\n"))
}
