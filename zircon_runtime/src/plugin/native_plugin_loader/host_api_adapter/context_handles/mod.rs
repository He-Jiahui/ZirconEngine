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
mod tests;
