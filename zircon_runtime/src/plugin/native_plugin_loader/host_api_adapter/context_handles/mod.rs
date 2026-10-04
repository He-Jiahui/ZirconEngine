//! 注册和桥接共用带世代的宿主句柄空间，避免旧句柄复用后误入另一类回调。
mod registry;

use std::sync::{Arc, OnceLock};

use zircon_runtime_interface::ZrRuntimePluginHandle;

pub(super) use registry::{
    HostContextRegistry, NativeHostApiV3Context, NativeHostApiV3RegistrationContextPin,
    NativeHostApiV4RegistrationContextPin, NativeHostRegistrationScopeState,
};

fn contexts() -> &'static HostContextRegistry<NativeHostApiV3Context> {
    static CONTEXTS: OnceLock<HostContextRegistry<NativeHostApiV3Context>> = OnceLock::new();
    CONTEXTS.get_or_init(HostContextRegistry::default)
}

pub(super) fn insert_context(context: NativeHostApiV3Context) -> u64 {
    contexts().insert(Arc::new(context))
}

pub(super) fn remove_context(raw_handle: u64) {
    contexts().remove(raw_handle);
}

pub(super) fn context_snapshot(raw_handle: u64) -> Option<Arc<NativeHostApiV3Context>> {
    contexts().get(raw_handle)
}

/// V3 注册入口取得带关闭租约的视图；解析句柄成功并不代表 scope 仍接受新调用。
pub(super) fn context_for(
    handle: ZrRuntimePluginHandle,
) -> Option<NativeHostApiV3RegistrationContextPin> {
    if !handle.is_valid() {
        return None;
    }
    match contexts().get(handle.raw()).as_deref()? {
        NativeHostApiV3Context::RegistrationV4(context) => {
            NativeHostApiV3RegistrationContextPin::new(context.v3_context())
        }
        NativeHostApiV3Context::BridgeCall(_) => None,
    }
}

/// V4 注册入口在调用期钉住策略和借用的注册表；设计上 scope 关闭后拒绝新租约。
pub(super) fn context_for_v4(
    handle: ZrRuntimePluginHandle,
) -> Option<NativeHostApiV4RegistrationContextPin> {
    if !handle.is_valid() {
        return None;
    }
    match contexts().get(handle.raw()).as_deref()? {
        NativeHostApiV3Context::RegistrationV4(context) => {
            NativeHostApiV4RegistrationContextPin::new(context.clone())
        }
        NativeHostApiV3Context::BridgeCall(_) => None,
    }
}

#[cfg(test)]
pub(super) use registry::{HostContextDirectoryMetrics, HOST_CONTEXT_PAGE_SLOTS};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
