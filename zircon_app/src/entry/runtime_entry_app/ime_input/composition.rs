use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_IME_CURSOR_HIDDEN_V1,
};

use super::super::converters::{byte_slice, usize_to_u32};

pub(super) fn ime_preedit_event(
    viewport: ZrRuntimeViewportHandle,
    value: &str,
    cursor: Option<(usize, usize)>,
) -> ZrRuntimeEventV1 {
    let (cursor_start, cursor_end) = cursor
        .map(|(start, end)| (usize_to_u32(start), usize_to_u32(end)))
        .unwrap_or((
            ZR_RUNTIME_IME_CURSOR_HIDDEN_V1,
            ZR_RUNTIME_IME_CURSOR_HIDDEN_V1,
        ));
    ZrRuntimeEventV1::ime_preedit(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport,
        byte_slice(value),
        cursor_start,
        cursor_end,
    )
}

pub(super) fn ime_commit_event(viewport: ZrRuntimeViewportHandle, value: &str) -> ZrRuntimeEventV1 {
    ZrRuntimeEventV1::ime_commit(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, byte_slice(value))
}
