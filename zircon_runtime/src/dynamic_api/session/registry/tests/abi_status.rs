use super::*;

#[test]
fn astra_life_a4_owned_status_survives_owner_tls_reuse_and_preserves_status_code() {
    let mut original = error_status("owner operation failed");
    original.code = ZrStatusCode::LimitExceeded.as_raw();
    let owned = unsafe { OwnedSessionStatus::capture(original) };
    let _ = error_status("next operation overwrote the TLS buffer");
    let status = owned.into_abi();
    assert_eq!(status.status_code(), ZrStatusCode::LimitExceeded);
    assert_eq!(
        unsafe { status.diagnostics.checked_slice(4096) }.unwrap(),
        b"owner operation failed"
    );
}
