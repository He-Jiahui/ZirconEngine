//! 把静态清单中的成熟度与能力状态投影为运行时类型，供 metadata 对照测试消费。
#[path = "metadata/statuses.rs"]
mod capability_statuses;
#[path = "metadata/entry.rs"]
mod entry;
#[path = "metadata/maturity.rs"]
mod maturity;
#[path = "metadata/types.rs"]
mod types;

pub(in crate::tests::manifest) use types::StaticSoundPluginMetadata;

pub(super) fn static_plugin_metadata(manifest: &str) -> StaticSoundPluginMetadata {
    entry::static_plugin_metadata(manifest)
}
