use std::sync::Arc;

use crate::core::{CoreError, PluginContext, PluginFactory, ServiceFactory, ServiceObject};

pub fn factory(
    builder: impl Fn(&crate::core::CoreHandle) -> Result<ServiceObject, CoreError>
        + Send
        + Sync
        + 'static,
) -> ServiceFactory {
    Arc::new(builder)
}

pub fn plugin_factory(
    builder: impl Fn(&PluginContext) -> Result<ServiceObject, CoreError> + Send + Sync + 'static,
) -> PluginFactory {
    Arc::new(builder)
}
