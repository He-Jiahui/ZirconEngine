use zircon_runtime_interface::{ZrByteSlice, ZrStatus, ZrStatusCode};

use std::fmt::Display;

pub(in crate::dynamic_api::session) fn unsupported_version() -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::UnsupportedVersion,
        ZrByteSlice::from_static(b"unsupported runtime ABI version"),
    )
}

pub(in crate::dynamic_api::session) fn invalid_argument(message: &'static [u8]) -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::InvalidArgument,
        ZrByteSlice::from_static(message),
    )
}

pub(in crate::dynamic_api::session) fn not_found(message: &'static [u8]) -> ZrStatus {
    ZrStatus::new(ZrStatusCode::NotFound, ZrByteSlice::from_static(message))
}

pub(in crate::dynamic_api::session) fn error_status(message: impl Display) -> ZrStatus {
    let message = message.to_string();
    if message.is_empty() {
        return ZrStatus::new(
            ZrStatusCode::Error,
            ZrByteSlice::from_static(b"runtime dynamic API error"),
        );
    }
    let bytes: &'static [u8] = Box::leak(message.into_bytes().into_boxed_slice());
    ZrStatus::new(
        ZrStatusCode::Error,
        ZrByteSlice {
            data: bytes.as_ptr(),
            len: bytes.len(),
        },
    )
}
