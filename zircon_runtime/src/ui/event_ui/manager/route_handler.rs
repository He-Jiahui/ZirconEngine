use std::sync::Arc;

use serde_json::Value;

use zircon_runtime_interface::ui::event_ui::{UiInvocationContext, UiInvocationError};

pub(super) type RouteHandler =
    Arc<dyn Fn(UiInvocationContext) -> Result<Value, UiInvocationError> + Send + Sync + 'static>;
