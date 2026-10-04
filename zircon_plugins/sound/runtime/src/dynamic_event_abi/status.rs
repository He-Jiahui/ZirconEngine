//! 回调返回的诊断仍属于提供方；宿主在下次 ABI 调用前限额复制，长度/空指针检查不负责证明原生内存可读。
use zircon_runtime_interface::{
    ZrByteSlice, ZrStatusCode, ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1,
};

pub(crate) fn status_detail(code: ZrStatusCode, diagnostics: ZrByteSlice) -> String {
    let diagnostics = match unsafe {
        diagnostics.checked_slice(ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1)
    } {
        Ok(diagnostics) => diagnostics,
        Err(error) => {
            return format!(
                "sound dynamic event callback returned {code:?} with invalid diagnostics: {error:?}"
            );
        }
    };
    if diagnostics.is_empty() {
        format!("sound dynamic event callback returned {code:?}")
    } else {
        String::from_utf8_lossy(diagnostics).into_owned()
    }
}
