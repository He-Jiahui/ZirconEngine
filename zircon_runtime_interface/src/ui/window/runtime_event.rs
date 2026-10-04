use crate::{
    ZrByteSliceError, ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZrRuntimeViewportMetricsV1,
    ZrRuntimeViewportSizeV1, ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1,
};

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
    /// Uses the supplied Rust bytes. The metadata's raw payload pointer is never read.
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

    /// Borrows the caller-owned ABI payload after checking its shape and size.
    ///
    /// # Safety
    ///
    /// For a nonempty payload the pointer must reference initialized, readable bytes
    /// that remain valid and are not mutated for `'payload`. Shape checks cannot prove
    /// pointer validity. An empty, null or over-budget carrier is checked before reading.
    pub unsafe fn from_abi(event: ZrRuntimeEventV1) -> Result<Self, ZrByteSliceError> {
        let payload = unsafe {
            event
                .payload
                .checked_slice(ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1)?
        };
        Self::new(event, payload)
    }

    pub fn payload(&self) -> &'payload [u8] {
        self.payload
    }
}
