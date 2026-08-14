use zircon_runtime_interface::ZrByteSlice;

use crate::core::framework::scene::SystemStage;

use super::*;

#[test]
fn native_host_api_adapter_reports_unknown_stage_with_typed_error() {
    let stage = SystemStage::ORDER.len() as u32;
    let error = NativeHostApiAdapterError::from(
        stage_from_abi(stage).expect_err("unknown host API stage should report typed decode error"),
    );

    assert!(matches!(
        &error,
        NativeHostApiAdapterError::AbiDecode {
            source: AbiDecodeError::UnknownSystemStage { stage: actual }
        } if *actual == stage
    ));
    assert_eq!(
        error.to_string(),
        format!("unknown native system stage {stage}")
    );
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn native_host_api_adapter_utf8_error_preserves_source() {
    let bytes = [0xff];
    let error = NativeHostApiAdapterError::from(unsafe {
        read_utf8(ZrByteSlice {
            data: bytes.as_ptr(),
            len: bytes.len(),
        })
        .expect_err("invalid host API byte slice should report typed UTF-8 decode error")
    });

    assert!(matches!(
        &error,
        NativeHostApiAdapterError::AbiDecode {
            source: AbiDecodeError::InvalidUtf8 { .. }
        }
    ));
    assert!(
        std::error::Error::source(&error).is_some(),
        "invalid UTF-8 adapter error should preserve Utf8Error source"
    );
}
