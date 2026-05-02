use std::ptr;

use zircon_runtime_interface::{
    ZrOwnedByteBuffer, ZrRuntimeFrameV1, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use crate::core::framework::render::CapturedFrame;

const RUNTIME_FRAME_BUFFER_OWNER_TOKEN: u64 = 0x5a52_4652_414d_4501;

pub(super) fn encode_frame(frame: CapturedFrame) -> ZrRuntimeFrameV1 {
    ZrRuntimeFrameV1 {
        abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
        width: frame.width,
        height: frame.height,
        generation: frame.generation,
        rgba: owned_rgba_buffer(frame.rgba),
    }
}

fn owned_rgba_buffer(mut rgba: Vec<u8>) -> ZrOwnedByteBuffer {
    if rgba.is_empty() {
        return ZrOwnedByteBuffer::empty();
    }
    let buffer = ZrOwnedByteBuffer {
        data: rgba.as_mut_ptr(),
        len: rgba.len(),
        capacity: rgba.capacity(),
        owner_token: RUNTIME_FRAME_BUFFER_OWNER_TOKEN,
        free: Some(free_runtime_frame_bytes),
    };
    std::mem::forget(rgba);
    buffer
}

pub(super) unsafe extern "C" fn free_runtime_frame_bytes(buffer: ZrOwnedByteBuffer) -> ZrStatus {
    if buffer.is_empty() {
        return ZrStatus::ok();
    }
    if buffer.owner_token != RUNTIME_FRAME_BUFFER_OWNER_TOKEN || buffer.data.is_null() {
        return ZrStatus::new(
            ZrStatusCode::InvalidArgument,
            invalid_frame_buffer_message(),
        );
    }
    let len = buffer.len;
    let capacity = buffer.capacity;
    if len > capacity {
        return ZrStatus::new(
            ZrStatusCode::InvalidArgument,
            invalid_frame_buffer_message(),
        );
    }
    // Reclaim the allocation with the original pointer, length, and capacity exported to the host.
    let _ = unsafe { Vec::from_raw_parts(buffer.data, len, capacity) };
    ZrStatus::ok()
}

fn invalid_frame_buffer_message() -> zircon_runtime_interface::ZrByteSlice {
    zircon_runtime_interface::ZrByteSlice::from_static(b"invalid runtime frame buffer")
}

pub(super) fn write_frame(destination: *mut ZrRuntimeFrameV1, frame: ZrRuntimeFrameV1) -> ZrStatus {
    if destination.is_null() {
        return ZrStatus::new(
            ZrStatusCode::InvalidArgument,
            zircon_runtime_interface::ZrByteSlice::from_static(b"missing frame output"),
        );
    }
    unsafe {
        ptr::write(destination, frame);
    }
    ZrStatus::ok()
}
