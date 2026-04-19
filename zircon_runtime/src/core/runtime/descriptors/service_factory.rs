use std::sync::Arc;

use super::super::handle::CoreHandle;
use crate::core::error::CoreError;
use crate::core::types::ServiceObject;

pub type ServiceFactory =
    Arc<dyn Fn(&CoreHandle) -> Result<ServiceObject, CoreError> + Send + Sync>;
