use std::ptr;

use zircon_runtime_interface::ui::accessibility::UiAccessibilityTreeSnapshot;
use zircon_runtime_interface::{ZrOwnedResultV2, ZrRuntimeFrameV2, ZrStatus, ZrStatusCode};

use crate::core::framework::render::CapturedFrame;

pub(super) struct EncodedRuntimeFrame {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) generation: u64,
    pub(super) rgba: Vec<u8>,
}

pub(super) fn encode_frame(frame: CapturedFrame) -> EncodedRuntimeFrame {
    EncodedRuntimeFrame {
        width: frame.width,
        height: frame.height,
        generation: frame.generation,
        rgba: frame.rgba,
    }
}

pub(super) fn encode_accessibility_tree(
    snapshot: &UiAccessibilityTreeSnapshot,
) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(snapshot)
}

pub(super) fn encode_profile_response<T: serde::Serialize>(
    response: &T,
) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(response)
}

pub(super) fn encode_host_request_batch<T: serde::Serialize>(
    response: &T,
) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(response)
}

pub(super) fn encode_world_sync_payload<T: serde::Serialize>(
    response: &T,
) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(response)
}

pub(super) fn write_frame(destination: *mut ZrRuntimeFrameV2, frame: ZrRuntimeFrameV2) -> ZrStatus {
    if destination.is_null() {
        return missing_output(b"missing frame output");
    }
    unsafe { ptr::write(destination, frame) };
    ZrStatus::ok()
}

pub(super) fn write_accessibility_tree(
    destination: *mut ZrOwnedResultV2,
    output: ZrOwnedResultV2,
) -> ZrStatus {
    write_output(destination, output, b"missing accessibility tree output")
}

pub(super) fn write_profile_response(
    destination: *mut ZrOwnedResultV2,
    output: ZrOwnedResultV2,
) -> ZrStatus {
    write_output(destination, output, b"missing profile output")
}

pub(super) fn write_host_requests(
    destination: *mut ZrOwnedResultV2,
    output: ZrOwnedResultV2,
) -> ZrStatus {
    write_output(destination, output, b"missing host request output")
}

pub(super) fn write_world_sync_payload(
    destination: *mut ZrOwnedResultV2,
    output: ZrOwnedResultV2,
) -> ZrStatus {
    write_output(destination, output, b"missing runtime world sync output")
}

fn write_output(
    destination: *mut ZrOwnedResultV2,
    output: ZrOwnedResultV2,
    missing_message: &'static [u8],
) -> ZrStatus {
    if destination.is_null() {
        return missing_output(missing_message);
    }
    unsafe { ptr::write(destination, output) };
    ZrStatus::ok()
}

fn missing_output(message: &'static [u8]) -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::InvalidArgument,
        zircon_runtime_interface::ZrByteSlice::from_static(message),
    )
}
