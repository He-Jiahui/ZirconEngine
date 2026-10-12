//! 将原生插件宿主 ABI 拆为解码、注册、句柄生命周期和桥接调度；外部只取得受控 scope。
#[path = "host_api_adapter/abi_decode/mod.rs"]
mod abi_decode;
#[path = "host_api_adapter/bridge_scope/mod.rs"]
mod bridge_scope;
#[path = "host_api_adapter/context_handles/mod.rs"]
mod context_handles;
#[path = "host_api_adapter/ecs_registration/mod.rs"]
mod ecs_registration;
#[path = "host_api_adapter/registration_policy/mod.rs"]
mod registration_policy;

pub use bridge_scope::NativeHostBridgeCallScope;
// TODO: [CR-PLUGIN-NATIVE-0103] 确认 V4 注册 scope 的生产调用时机；仓内目前仅测试构造该 scope，缺少真实加载路径的调用证据；下一步追踪插件入口与运行时扩展注册衔接。
pub use registration_policy::{
    NativeHostApiV4RegistrationPolicy, NativeHostApiV4RegistrationScope,
};
