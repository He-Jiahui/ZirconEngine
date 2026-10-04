use crate::{
    ui::window::{
        runtime_abi_event_to_window_input_pump_event, runtime_event_to_window_input_pump_event,
        UiRuntimeEvent, UiRuntimeEventAdapterContext, UiRuntimeEventAdapterError,
    },
    ZrByteSlice, ZrByteSliceError, ZrRuntimeEventV1, ZrRuntimeViewportHandle,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1,
    ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1, ZR_RUNTIME_KEY_ACTION_TEXT_V1,
};

fn text_event() -> ZrRuntimeEventV1 {
    let mut event = ZrRuntimeEventV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1,
        ZrRuntimeViewportHandle::new(1),
    );
    event.button = ZR_RUNTIME_KEY_ACTION_TEXT_V1;
    event
}

#[test]
fn safe_event_uses_borrowed_bytes_and_never_reads_raw_metadata_pointer() {
    let mut metadata = text_event();
    metadata.payload = ZrByteSlice {
        data: usize::MAX as *const u8,
        len: 128,
    };
    let payload = String::from("input 文本");
    let event = UiRuntimeEvent::new(metadata, payload.as_bytes()).unwrap();
    assert_eq!(event.payload(), payload.as_bytes());
    let context = UiRuntimeEventAdapterContext::for_window("test");
    let safe_output = runtime_event_to_window_input_pump_event(&context, event).unwrap();

    metadata.payload = ZrByteSlice {
        data: payload.as_ptr(),
        len: payload.len(),
    };
    // The fixture owns the complete payload and keeps it immutable through conversion.
    let abi_output =
        unsafe { runtime_abi_event_to_window_input_pump_event(&context, metadata) }.unwrap();
    assert_eq!(safe_output, abi_output);
}

#[test]
fn safe_event_is_send_and_sync_and_checks_the_payload_budget() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<UiRuntimeEvent<'_>>();
    let bytes = vec![0; ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1 + 1];
    assert_eq!(
        UiRuntimeEvent::new(text_event(), &bytes).unwrap_err(),
        ZrByteSliceError::LengthExceedsLimit {
            len: bytes.len(),
            limit: ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1,
        },
    );
}

#[test]
fn abi_event_rejects_invalid_shape_and_version_before_pointer_reads() {
    let context = UiRuntimeEventAdapterContext::for_window("test");
    let mut metadata = text_event();
    metadata.payload = ZrByteSlice {
        data: core::ptr::null(),
        len: 1,
    };
    // The null carrier is rejected without dereferencing it.
    assert_eq!(
        unsafe { runtime_abi_event_to_window_input_pump_event(&context, metadata) }.unwrap_err(),
        UiRuntimeEventAdapterError::InvalidTextPayload,
    );
    metadata.abi_version += 1;
    metadata.payload.data = usize::MAX as *const u8;
    assert!(matches!(
        // An unsupported version is rejected before decoding any payload carrier.
        unsafe { runtime_abi_event_to_window_input_pump_event(&context, metadata) },
        Err(UiRuntimeEventAdapterError::UnsupportedAbi { .. })
    ));
}
