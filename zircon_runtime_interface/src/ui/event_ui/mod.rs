//! UI 事件控制与反射数据的公共出口。
//!
//! 这里只挂载绑定文本适配、控制消息与两类反射快照；消息和快照拥有各自的数据。
//! 路由处理器、反射存储及通知接收器的生命周期由 Runtime/Editor 消费方管理，
//! 导出这些类型本身不会注册路由、建立订阅或执行属性更新。

#[path = "codec.rs"]
mod codec;
#[path = "control.rs"]
mod control;
#[path = "reflection.rs"]
mod reflection;

pub use codec::UiBindingCodec;
pub use control::{
    UiControlRequest, UiControlResponse, UiInvocationContext, UiInvocationError,
    UiInvocationRequest, UiInvocationResponse, UiInvocationResult, UiInvocationSource,
    UiNotification, UiRouteId, UiSubscriptionId,
};
pub use reflection::{
    UiActionDescriptor, UiNodeDescriptor, UiNodeId, UiNodePath, UiParameterDescriptor,
    UiPropertyDescriptor, UiPropertyInvalidationReason, UiReflectedProperty,
    UiReflectedPropertySource, UiReflectionDiff, UiReflectionNodePatch, UiReflectionSnapshot,
    UiReflectorHitContext, UiReflectorNode, UiReflectorSnapshot, UiStateFlags, UiTreeId,
    UiValueType, UiWidgetLifecycleState,
};
