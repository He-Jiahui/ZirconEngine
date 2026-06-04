use zircon_runtime_interface::{ZrByteSlice, ZrStatus, ZrStatusCode};

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

pub(in crate::dynamic_api::session) fn error_status(_message: impl Into<String>) -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::Error,
        ZrByteSlice::from_static(b"runtime dynamic API error"),
    )
}
