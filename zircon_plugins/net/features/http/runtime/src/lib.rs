mod backend;
mod feature;

pub use backend::{http_runtime_backend, HyperReqwestHttpBackend};
pub use feature::{
    feature_manifest, http_runtime_manager, module_descriptor, plugin_feature_registration,
    runtime_plugin_feature, NetHttpRuntimeFeature, NET_HTTP_FEATURE_CAPABILITY,
    NET_HTTP_FEATURE_ID, NET_HTTP_FEATURE_MANAGER_NAME, NET_HTTP_FEATURE_MODULE_NAME,
};

#[cfg(test)]
mod tests;
