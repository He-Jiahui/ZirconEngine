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
