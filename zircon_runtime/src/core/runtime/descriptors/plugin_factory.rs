use std::sync::Arc;

use super::super::contexts::PluginContext;
use super::ServiceObject;
use crate::core::CoreError;

pub type PluginFactory =
    Arc<dyn Fn(&PluginContext) -> Result<ServiceObject, CoreError> + Send + Sync>;
