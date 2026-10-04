//! 集中公开内容下载 feature、状态 manager 与能力常量；宿主通过 feature 注册，测试可单独构造内存 manager。

mod capability;
mod feature;
mod manager;
mod plugin;

pub use capability::{NET_CONTENT_DOWNLOAD_FEATURE_CAPABILITY, RUNTIME_CAPABILITIES};
pub use manager::{net_content_download_runtime_manager, NetContentDownloadRuntimeManager};
pub use plugin::{
    feature_manifest, module_descriptor, plugin_feature_registration, runtime_plugin_feature,
    NetContentDownloadRuntimeFeature, NET_CONTENT_DOWNLOAD_FEATURE_ID,
    NET_CONTENT_DOWNLOAD_FEATURE_MANAGER_NAME, NET_CONTENT_DOWNLOAD_FEATURE_MODULE_NAME,
};

#[cfg(test)]
mod tests;
