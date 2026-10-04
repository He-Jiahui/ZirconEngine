use zircon_runtime_interface::{
    ZrRuntimeImeHostRequestV1, ZrRuntimeViewportHandle, ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
};

use super::ime_request_targets_viewport;

#[test]
fn ime_request_rejects_a_different_viewport() {
    let request =
        ZrRuntimeImeHostRequestV1::enable().with_target_viewport(ZrRuntimeViewportHandle::new(7));

    assert!(ime_request_targets_viewport(
        &request,
        ZrRuntimeViewportHandle::new(7)
    ));
    assert!(!ime_request_targets_viewport(
        &request,
        ZrRuntimeViewportHandle::new(8)
    ));
}

#[test]
fn legacy_ime_request_without_a_target_only_targets_the_default_viewport() {
    assert!(ime_request_targets_viewport(
        &ZrRuntimeImeHostRequestV1::enable(),
        ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1
    ));
    assert!(!ime_request_targets_viewport(
        &ZrRuntimeImeHostRequestV1::enable(),
        ZrRuntimeViewportHandle::new(7)
    ));
}
