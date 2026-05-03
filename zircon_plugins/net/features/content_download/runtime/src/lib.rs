mod feature;
mod manager;

pub use feature::{
    feature_manifest, module_descriptor, plugin_feature_registration, runtime_plugin_feature,
    NetContentDownloadRuntimeFeature, NET_CONTENT_DOWNLOAD_FEATURE_CAPABILITY,
    NET_CONTENT_DOWNLOAD_FEATURE_ID, NET_CONTENT_DOWNLOAD_FEATURE_MANAGER_NAME,
    NET_CONTENT_DOWNLOAD_FEATURE_MODULE_NAME,
};
pub use manager::{net_content_download_runtime_manager, NetContentDownloadRuntimeManager};

#[cfg(test)]
mod tests;
