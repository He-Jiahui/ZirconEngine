use std::sync::Arc;

use crate::core::runtime::ServiceObject;
use crate::core::{CoreError, PluginContext, PluginFactory, ServiceFactory};

pub fn factory(
    builder: impl Fn(&crate::core::CoreWeak) -> Result<ServiceObject, CoreError> + Send + Sync + 'static,
) -> ServiceFactory {
    Arc::new(builder)
}

pub fn plugin_factory(
    builder: impl Fn(&PluginContext) -> Result<ServiceObject, CoreError> + Send + Sync + 'static,
) -> PluginFactory {
    Arc::new(builder)
}
