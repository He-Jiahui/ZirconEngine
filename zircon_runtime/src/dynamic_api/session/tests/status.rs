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
