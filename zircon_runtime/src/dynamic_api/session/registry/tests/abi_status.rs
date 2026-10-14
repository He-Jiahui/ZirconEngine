//! 核对诊断在 TLS 覆盖前复制，接收线程重建时仍保留原状态码和字节。

use super::*;

#[test]
/// 同线程模拟 owner 的下一次错误覆盖原 TLS，OwnedSessionStatus 必须保留前一次诊断。
fn astra_life_a4_owned_status_survives_owner_tls_reuse_and_preserves_status_code() {
    let mut original = error_status("owner operation failed");
    original.code = ZrStatusCode::LimitExceeded.as_raw();
    // SAFETY: original 刚由本线程 error_status 构造，复制前尚无覆盖其 TLS 的调用。
    let owned = unsafe { OwnedSessionStatus::capture(original) };
    let _ = error_status("next operation overwrote the TLS buffer");
    let status = owned.into_abi();
    assert_eq!(status.status_code(), ZrStatusCode::LimitExceeded);
    // SAFETY: into_abi 刚在本线程重建诊断；在读取前没有下一次覆盖调用，长度也在预算内。
    assert_eq!(
        unsafe { status.diagnostics.checked_slice(4096) }.unwrap(),
        b"owner operation failed"
    );
}
