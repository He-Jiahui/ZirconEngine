// 仅汇总 metadata 测试需要的两类字段；静态文件是输入，运行时描述符是独立对照源。
use super::capability_statuses::capability_statuses_from_plugin_toml;
use super::maturity::static_maturity_from_plugin_toml;
use super::types::StaticSoundPluginMetadata;

pub(super) fn static_plugin_metadata(manifest: &str) -> StaticSoundPluginMetadata {
    StaticSoundPluginMetadata {
        maturity: static_maturity_from_plugin_toml(manifest),
        capability_statuses: capability_statuses_from_plugin_toml(manifest),
    }
}
