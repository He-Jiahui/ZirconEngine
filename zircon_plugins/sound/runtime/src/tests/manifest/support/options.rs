// 选项静态投影同时供键集合与完整元数据测试使用；生产源由 sound_options 提供。
mod keys;
mod parser;
mod projection;
mod state;

pub(super) use keys::option_keys_from_plugin_toml;
pub(super) use parser::option_manifests_from_plugin_toml;
pub(super) use projection::option_manifest_tuple;
