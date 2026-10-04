//! V4 注册 scope 组合宿主授权策略与回调表；真正的 ECS 变更由注册入口模块完成。
mod context;
mod policy;
mod scope;

pub use policy::NativeHostApiV4RegistrationPolicy;
pub use scope::NativeHostApiV4RegistrationScope;

pub(super) use context::NativeHostApiV4RegistrationContext;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
