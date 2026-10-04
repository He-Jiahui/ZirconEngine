use std::sync::Arc;

use serde_json::Value;

use zircon_runtime_interface::ui::event_ui::{UiInvocationContext, UiInvocationError};

// Arc 让路由表持有宿主注册的可共享处理器；业务状态应通过处理器捕获的 owner 访问，避免复制到反射快照。
pub(super) type RouteHandler =
    Arc<dyn Fn(UiInvocationContext) -> Result<Value, UiInvocationError> + Send + Sync + 'static>;
