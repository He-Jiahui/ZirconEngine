use std::sync::Arc;

use super::super::weak::CoreWeak;
use super::ServiceObject;
use crate::core::CoreError;

pub type ServiceFactory = Arc<dyn Fn(&CoreWeak) -> Result<ServiceObject, CoreError> + Send + Sync>;
