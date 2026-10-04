use zircon_runtime_interface::{
    ZrStatus, ZrStatusCode, ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1,
};

use crate::dynamic_api::session::status::error_status;

/// An owned message replaces the ABI diagnostic pointer before leaving its TLS owner.
#[derive(Clone, Debug)]
pub(in crate::dynamic_api::session) struct OwnedSessionStatus {
    code: ZrStatusCode,
    diagnostics: String,
}

impl OwnedSessionStatus {
    /// # Safety
    /// The status must come from a completed runtime call on the current thread;
    /// its diagnostics must remain readable until this function returns.
    pub(in crate::dynamic_api::session) unsafe fn capture(status: ZrStatus) -> Self {
        let diagnostics = unsafe {
            status
                .diagnostics
                .checked_slice(ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1)
        }
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .unwrap_or_else(|_| "runtime produced invalid status diagnostics".to_owned());
        Self {
            code: status.status_code(),
            diagnostics,
        }
    }

    pub(in crate::dynamic_api::session) fn into_abi(self) -> ZrStatus {
        if self.code == ZrStatusCode::Ok {
            return ZrStatus::ok();
        }
        let mut status = error_status(self.diagnostics);
        status.code = self.code.as_raw();
        status
    }

    pub(in crate::dynamic_api::session) fn into_message(self) -> String {
        self.diagnostics
    }
}

#[cfg(test)]
#[path = "tests/abi_status.rs"]
mod tests;
