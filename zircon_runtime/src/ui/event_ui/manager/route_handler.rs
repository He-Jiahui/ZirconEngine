use std::sync::Arc;

use serde_json::Value;

use super::super::{UiInvocationContext, UiInvocationError};

pub(super) type RouteHandler =
    Arc<dyn Fn(UiInvocationContext) -> Result<Value, UiInvocationError> + Send + Sync + 'static>;
