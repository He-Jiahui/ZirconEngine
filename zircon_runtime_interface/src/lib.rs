//! Stable ABI and DTO contracts shared by runtime hosts, editors, and plugins.

pub mod buffer;
pub mod handles;
pub mod manifest;
pub mod math;
pub mod plugin_api;
pub mod resource;
pub mod runtime_api;
pub mod status;
pub mod ui;
pub mod version;

pub use buffer::{ZrByteSlice, ZrFreeBytesFn, ZrOwnedByteBuffer};
pub use handles::{ZrRuntimePluginHandle, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle};
pub use manifest::{ZrPluginModuleDescriptorV1, ZrPluginModuleKind, ZrRuntimeTargetMode};
pub use plugin_api::{
    ZrPluginApiV1, ZrPluginEntryFnV1, ZrPluginEntryReportV1, ZR_PLUGIN_ENTRY_SYMBOL_V1,
};
pub use runtime_api::{
    ZrHostApiV1, ZrRuntimeApiV1, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1,
    ZrRuntimeGetApiFnV1, ZrRuntimeHostFetchFnV1, ZrRuntimeHostFetchRequestV1,
    ZrRuntimeSessionConfigV1, ZrRuntimeTranslatedEventV1, ZrRuntimeViewportMetricsV1,
    ZrRuntimeViewportSizeV1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1,
    ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1, ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1,
    ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1, ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1,
    ZR_RUNTIME_EVENT_KIND_TOUCH_V1, ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1,
    ZR_RUNTIME_FETCH_FLAG_STREAMING_V1, ZR_RUNTIME_GET_API_SYMBOL_V1,
    ZR_RUNTIME_KEY_ACTION_PRESSED_V1, ZR_RUNTIME_KEY_ACTION_RELEASED_V1,
    ZR_RUNTIME_KEY_ACTION_TEXT_V1, ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1, ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_RESUMED_V1, ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1,
    ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1, ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1,
    ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1, ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1,
    ZR_RUNTIME_TOUCH_PHASE_ENDED_V1, ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
    ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
};
pub use status::{ZrStatus, ZrStatusCode};
pub use version::ZIRCON_RUNTIME_ABI_VERSION_V1;

#[cfg(test)]
mod tests;
