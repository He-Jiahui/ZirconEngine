//! Runtime registry service object storage slot.

use std::any::Any;
use std::sync::Arc;

pub type ServiceObject = Arc<dyn Any + Send + Sync>;
