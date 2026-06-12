use std::sync::Arc;

use super::super::handle::CoreHandle;
use super::ServiceObject;
use crate::core::CoreError;

pub type ServiceFactory =
    Arc<dyn Fn(&CoreHandle) -> Result<ServiceObject, CoreError> + Send + Sync>;
