use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1, ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1,
};

use super::admission::RuntimeImeInputAdmission;

pub(super) const fn ime_enabled_event(viewport: ZrRuntimeViewportHandle) -> ZrRuntimeEventV1 {
    ZrRuntimeEventV1::ime_enabled(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport)
}

pub(super) const fn ime_disabled_event(viewport: ZrRuntimeViewportHandle) -> ZrRuntimeEventV1 {
    ZrRuntimeEventV1::ime_disabled(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport)
}

pub(in crate::entry::runtime_entry_app) fn focus_changed_events(
    admission: &mut RuntimeImeInputAdmission,
    viewport: ZrRuntimeViewportHandle,
    focused: bool,
) -> [Option<ZrRuntimeEventV1>; 2] {
    focus_changed_events_with_native_source(admission, viewport, focused, false)
}

pub(in crate::entry::runtime_entry_app) fn focus_changed_events_with_native_source(
    admission: &mut RuntimeImeInputAdmission,
    viewport: ZrRuntimeViewportHandle,
    focused: bool,
    native_source_active: bool,
) -> [Option<ZrRuntimeEventV1>; 2] {
    let cancel = admission
        .set_window_focused(focused)
        .then_some(())
        .filter(|_| !native_source_active)
        .map(|_| ime_disabled_event(viewport));
    let state = if focused {
        ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1
    } else {
        ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1
    };
    [
        cancel,
        Some(ZrRuntimeEventV1::lifecycle(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            state,
        )),
    ]
}
