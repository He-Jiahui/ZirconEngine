use std::cell::RefCell;
use std::fmt::{self, Display, Write as _};

use zircon_runtime_interface::{
    ZrByteSlice, ZrStatus, ZrStatusCode, ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1,
};

use crate::dynamic_api::bounded_json::BoundedJsonError;

thread_local! {
    static STATUS_DIAGNOSTICS: RefCell<StatusDiagnosticsBuffer> =
        const { RefCell::new(StatusDiagnosticsBuffer::new()) };
}

struct StatusDiagnosticsBuffer {
    bytes: [u8; ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1],
    len: usize,
}

impl StatusDiagnosticsBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1],
            len: 0,
        }
    }
}

impl fmt::Write for StatusDiagnosticsBuffer {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let available = self.bytes.len().saturating_sub(self.len);
        let mut count = available.min(value.len());
        while count > 0 && !value.is_char_boundary(count) {
            count -= 1;
        }
        self.bytes[self.len..self.len + count].copy_from_slice(&value.as_bytes()[..count]);
        self.len += count;
        Ok(())
    }
}

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

pub(in crate::dynamic_api::session) fn limit_exceeded(message: &'static [u8]) -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::LimitExceeded,
        ZrByteSlice::from_static(message),
    )
}

pub(in crate::dynamic_api::session) fn invalid_or_limit_payload(
    error: &BoundedJsonError,
    invalid_message: &'static [u8],
    limit_message: &'static [u8],
) -> ZrStatus {
    if error.is_limit_exceeded() {
        limit_exceeded(limit_message)
    } else {
        invalid_argument(invalid_message)
    }
}

pub(in crate::dynamic_api::session) fn output_payload_status(
    error: BoundedJsonError,
    limit_message: &'static [u8],
) -> ZrStatus {
    if error.is_limit_exceeded() {
        limit_exceeded(limit_message)
    } else {
        error_status(error)
    }
}

pub(in crate::dynamic_api::session) fn not_found(message: &'static [u8]) -> ZrStatus {
    ZrStatus::new(ZrStatusCode::NotFound, ZrByteSlice::from_static(message))
}

pub(in crate::dynamic_api::session) fn teardown_incomplete() -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::Error,
        ZrByteSlice::from_static(b"runtime session resources did not quiesce before unload"),
    )
}

pub(in crate::dynamic_api::session) fn error_status(message: impl Display) -> ZrStatus {
    STATUS_DIAGNOSTICS.with(|storage| {
        let mut storage = storage.borrow_mut();
        storage.len = 0;
        let _ = storage.write_fmt(format_args!("{message}"));
        if storage.len == 0 {
            return ZrStatus::new(
                ZrStatusCode::Error,
                ZrByteSlice::from_static(b"runtime dynamic API error"),
            );
        }
        ZrStatus::new(
            ZrStatusCode::Error,
            ZrByteSlice {
                data: storage.bytes.as_ptr(),
                len: storage.len,
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1;

    use super::error_status;

    #[test]
    fn dynamic_status_diagnostics_are_bounded_and_utf8_aligned() {
        let status = error_status("界".repeat(ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1));
        let diagnostics = unsafe {
            status
                .diagnostics
                .checked_slice(ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1)
        }
        .expect("bounded status diagnostics");

        assert_eq!(diagnostics.len(), 4_095);
        assert!(std::str::from_utf8(diagnostics).is_ok());
    }
}
