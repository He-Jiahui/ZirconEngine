use zircon_runtime_interface::{ZrByteSlice, ZrStatusCode};

pub(crate) fn status_detail(code: ZrStatusCode, diagnostics: ZrByteSlice) -> String {
    let diagnostics = unsafe { diagnostics.as_slice() };
    if diagnostics.is_empty() {
        format!("sound dynamic event callback returned {code:?}")
    } else {
        String::from_utf8_lossy(diagnostics).into_owned()
    }
}
