//! 公开 HTTP feature 注册入口与可注入后端，供根 net manager 激活真实 socket HTTP。
//! 独立测试构造器会创建单独 manager，生产 factory 将后端安装到已注册根服务。

mod backend;
mod capability;
mod feature;
mod plugin;

pub use backend::{http_runtime_backend, HyperReqwestHttpBackend};
pub use capability::{NET_HTTP_FEATURE_CAPABILITY, RUNTIME_CAPABILITIES};
pub use plugin::{
    feature_manifest, http_runtime_manager, module_descriptor, plugin_feature_registration,
    runtime_plugin_feature, NetHttpRuntimeFeature, NET_HTTP_FEATURE_ID,
    NET_HTTP_FEATURE_MANAGER_NAME, NET_HTTP_FEATURE_MODULE_NAME,
};

#[cfg(test)]
mod tests;
