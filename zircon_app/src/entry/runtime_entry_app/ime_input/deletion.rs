use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::super::converters::usize_to_u32;

pub(super) fn ime_delete_surrounding_event(
    viewport: ZrRuntimeViewportHandle,
    before_bytes: usize,
    after_bytes: usize,
) -> ZrRuntimeEventV1 {
    ZrRuntimeEventV1::ime_delete_surrounding(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport,
        usize_to_u32(before_bytes),
        usize_to_u32(after_bytes),
    )
}
