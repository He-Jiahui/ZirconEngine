//! 固定 Editor 网络 authoring 能力名，与运行时 net 插件共用包 ID。
//! Editor 注册和 UI 能力过滤消费该名称；它不表示 HTTP/WS 后端已经激活。

pub const PLUGIN_ID: &str = zircon_plugin_net_runtime::PLUGIN_ID;
pub const NET_AUTHORING_CAPABILITY: &str = "editor.extension.net_authoring";

pub const EDITOR_CAPABILITIES: &[&str] = &[NET_AUTHORING_CAPABILITY];
