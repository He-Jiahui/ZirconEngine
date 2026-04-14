//! Runtime UI module skeleton wired into the core runtime.

mod binding;
mod event_ui;

use zircon_module::{stub_module_descriptor, ModuleDescriptor};

pub use binding::{
    UiBindingCall, UiBindingParseError, UiBindingValue, UiEventBinding, UiEventKind, UiEventPath,
    UiEventRouter,
};
pub use event_ui::{
    UiActionDescriptor, UiBindingCodec, UiControlRequest, UiControlResponse, UiEventManager,
    UiInvocationContext, UiInvocationError, UiInvocationRequest, UiInvocationResponse,
    UiInvocationResult, UiInvocationSource, UiNodeDescriptor, UiNodeId, UiNodePath, UiNotification,
    UiParameterDescriptor, UiPropertyDescriptor, UiReflectionDiff, UiReflectionSnapshot, UiRouteId,
    UiStateFlags, UiSubscriptionId, UiTreeId, UiValueType,
};

pub const UI_MODULE_NAME: &str = "UiModule";

#[derive(Clone, Debug, Default)]
pub struct UiConfig {
    pub enabled: bool,
}

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
        UI_MODULE_NAME,
        "Runtime UI widgets and layout",
        "UiDriver",
        "UiManager",
    )
}

#[cfg(test)]
mod tests;
