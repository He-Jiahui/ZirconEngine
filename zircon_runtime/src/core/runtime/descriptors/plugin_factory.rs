use std::sync::Arc;

use super::super::contexts::PluginContext;
use crate::core::error::CoreError;
use crate::core::types::ServiceObject;

pub type PluginFactory =
    Arc<dyn Fn(&PluginContext) -> Result<ServiceObject, CoreError> + Send + Sync>;
