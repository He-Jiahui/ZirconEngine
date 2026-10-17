use crate::{
    ZrByteSliceError, ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZrRuntimeViewportMetricsV1,
    ZrRuntimeViewportSizeV1, ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1,
};

/// 这是 Rust 借用视图，不是 C ABI 载体；复制视图不会延长载荷有效期或转移释放责任。
/// An input event whose payload is a valid Rust borrow for the complete conversion.
/// Raw ABI pointers are copied into a borrow only by [`Self::from_abi`].
///
/// ```compile_fail
/// use zircon_runtime_interface::{ui::window::UiRuntimeEvent, ZrRuntimeEventV1, ZrRuntimeViewportHandle};
/// let event;
/// {
///     let text = String::from("input");
///     let metadata = ZrRuntimeEventV1::new(1, 0, ZrRuntimeViewportHandle::new(1));
///     event = UiRuntimeEvent::new(metadata, text.as_bytes()).unwrap();
/// }
/// let _ = event.payload();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiRuntimeEvent<'payload> {
    pub(super) abi_version: u32,
    pub(super) kind: u32,
    pub(super) viewport: ZrRuntimeViewportHandle,
    pub(super) size: ZrRuntimeViewportSizeV1,
    pub(super) metrics: ZrRuntimeViewportMetricsV1,
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) delta: f32,
    pub(super) button: u32,
    pub(super) state: u32,
    pub(super) pointer_id: u64,
    pub(super) key_code: u32,
    pub(super) scan_code: u32,
    pub(super) payload: &'payload [u8],
}

impl<'payload> UiRuntimeEvent<'payload> {
    /// 以调用方的 Rust 切片建立载荷视图，不读取元数据中的原始载荷指针。
    /// Uses the supplied Rust bytes. The metadata's raw payload pointer is never read.
    ///
    /// # 参数与返回
    /// `metadata` 只提供标量元数据，`payload` 是调用方已持有的有效切片；只检查切片长度上限。
    /// 超限返回长度错误，否则返回与该切片同生命周期的视图；不校验 ABI 版本、种类或内容。
    pub fn new(
        metadata: ZrRuntimeEventV1,
        payload: &'payload [u8],
    ) -> Result<Self, ZrByteSliceError> {
        if payload.len() > ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1 {
            return Err(ZrByteSliceError::LengthExceedsLimit {
                len: payload.len(),
                limit: ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1,
            });
        }
        Ok(Self {
            abi_version: metadata.abi_version,
            kind: metadata.kind,
            viewport: metadata.viewport,
            size: metadata.size,
            metrics: metadata.metrics,
            x: metadata.x,
            y: metadata.y,
            delta: metadata.delta,
            button: metadata.button,
            state: metadata.state,
            pointer_id: metadata.pointer_id,
            key_code: metadata.key_code,
            scan_code: metadata.scan_code,
            payload,
        })
    }

    /// 检查载体形状与长度后借用 ABI 载荷，内存有效性仍由调用方保证。
    /// Borrows the caller-owned ABI payload after checking its shape and size.
    ///
    /// # Safety
    ///
    /// For a nonempty payload the pointer must reference initialized, readable bytes
    /// that remain valid and are not mutated for `'payload`. Shape checks cannot prove
    /// pointer validity. An empty, null or over-budget carrier is checked before reading.
    /// 非空字节还须位于同一有效分配中；地址范围、空指针和预算检查不能证明内存可读。
    /// 返回值借用原载荷，不分配、不复制或释放载荷；ABI 版本与内容检查由后续转换承担。
    pub unsafe fn from_abi(event: ZrRuntimeEventV1) -> Result<Self, ZrByteSliceError> {
        // SAFETY: 调用方保证会被借用的非空字节位于同一有效分配、已初始化且地址范围不回绕，
        // 并在返回视图的载荷借用期内保持可读、不变；形状和预算检查不能替代此前提。
        let payload = unsafe {
            event
                .payload
                .checked_slice(ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1)?
        };
        Self::new(event, payload)
    }

    /// 返回构造时的载荷借用；其有效期由原始所有者保证，与视图是否被复制无关。
    pub fn payload(&self) -> &'payload [u8] {
        self.payload
    }
}
