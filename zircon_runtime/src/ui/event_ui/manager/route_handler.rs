use std::sync::Arc;

use serde_json::Value;

use zircon_runtime_interface::ui::event_ui::{UiInvocationContext, UiInvocationError};

// Arc 让路由表持有宿主注册的可共享处理器；业务状态应通过处理器捕获的 owner 访问，避免复制到反射快照。
/// 同步接收拥有的调用上下文，返回 JSON 值或调用错误的共享回调。
/// 调用入口借用回调执行；回调及其捕获的存续由持有它的 Arc 共同决定。
pub(super) type RouteHandler =
    Arc<dyn Fn(UiInvocationContext) -> Result<Value, UiInvocationError> + Send + Sync + 'static>;
