//! 窗口 Runtime 事件的借用视图、原始 ABI 入口与提前拒绝顺序的对照测试。
//! 有效载荷由夹具保活；故意无效的载体只用于已核对的拒绝分支。

use crate::{
    ui::window::{
        runtime_abi_event_to_window_input_pump_event, runtime_event_to_window_input_pump_event,
        UiRuntimeEvent, UiRuntimeEventAdapterContext, UiRuntimeEventAdapterError,
    },
    ZrByteSlice, ZrByteSliceError, ZrRuntimeEventV1, ZrRuntimeViewportHandle,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1,
    ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1, ZR_RUNTIME_KEY_ACTION_TEXT_V1,
};

/// 构造尚未绑定载荷的文本事件元数据，供两个入口分别接收各自的字节来源。
fn text_event() -> ZrRuntimeEventV1 {
    let mut event = ZrRuntimeEventV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1,
        ZrRuntimeViewportHandle::new(1),
    );
    event.button = ZR_RUNTIME_KEY_ACTION_TEXT_V1;
    event
}

/// 对照独立借用与同一缓冲区的 ABI 解码，确认安全入口忽略元数据中的裸指针。
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
    // SAFETY: payload 的完整 String 字节在本作用域内保持初始化、有效且不变，长度取自同一分配。
    // ABI 转换同步完成，结果拥有数据，不保留原始载荷借用。
    let abi_output =
        unsafe { runtime_abi_event_to_window_input_pump_event(&context, metadata) }.unwrap();
    assert_eq!(safe_output, abi_output);
}

/// 用泛型约束检查借用视图的 Send/Sync，并独立核对超过载荷预算时的具体错误。
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

/// 固定当前拒绝顺序：合法版本拒绝空载荷形状，错误版本先于载荷解码返回。
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
        // SAFETY: 空指针与非零长度在载体形状检查中被拒绝，未构造或读取非空切片。
        unsafe { runtime_abi_event_to_window_input_pump_event(&context, metadata) }.unwrap_err(),
        UiRuntimeEventAdapterError::InvalidTextPayload,
    );
    metadata.abi_version += 1;
    metadata.payload.data = usize::MAX as *const u8;
    assert!(matches!(
        // An unsupported version is rejected before decoding any payload carrier.
        // SAFETY: 错误 ABI 版本在建立载荷借用前返回，哨兵地址不会进入读取分支。
        unsafe { runtime_abi_event_to_window_input_pump_event(&context, metadata) },
        Err(UiRuntimeEventAdapterError::UnsupportedAbi { .. })
    ));
}
