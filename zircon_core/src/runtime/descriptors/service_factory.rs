use std::sync::Arc;

use super::super::handle::CoreHandle;
use crate::error::CoreError;
use crate::types::ServiceObject;

pub type ServiceFactory =
    Arc<dyn Fn(&CoreHandle) -> Result<ServiceObject, CoreError> + Send + Sync>;
