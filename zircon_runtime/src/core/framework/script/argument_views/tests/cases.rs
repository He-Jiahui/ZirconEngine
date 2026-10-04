use super::super::{ScriptHostError, ScriptHostHotPathMetrics, ScriptHostValue};
use super::*;

#[test]
fn owned_argument_source_lends_text_and_bytes_without_generic_transport_clones() {
    let values = [
        ScriptHostValue::String("borrowed".to_string()),
        ScriptHostValue::Bytes(vec![7, 0, 128, 255]),
    ];
    let source = ScriptHostOwnedArgumentSource::new(&values);
    let arguments = ScriptHostArguments::new(&source);

    let text_length = arguments
        .with_argument(0, |value| match value {
            ScriptHostValueRef::String(value) => Ok(value.len()),
            value => Err(ScriptHostError::new(format!(
                "unexpected {:?}",
                value.kind()
            ))),
        })
        .unwrap();
    assert_eq!(text_length, "borrowed".len());

    let checksum = arguments
        .with_argument(1, |value| match value {
            ScriptHostValueRef::Bytes(value) => {
                let mut checksum = 0u32;
                for index in 0..value.len()? {
                    checksum += u32::from(value.byte_at(index)?);
                }
                Ok(checksum)
            }
            value => Err(ScriptHostError::new(format!(
                "unexpected {:?}",
                value.kind()
            ))),
        })
        .unwrap();
    assert_eq!(checksum, 390);
}

#[test]
fn explicit_owned_argument_conversions_record_only_their_business_boundary_copies() {
    let before = ScriptHostHotPathMetrics::snapshot();

    let text = ScriptHostValueRef::String("copy-at-boundary")
        .copy_string_at_business_boundary(0)
        .expect("string conversion should be valid");
    let bytes = ScriptHostValueRef::Bytes(ScriptHostByteView::Slice(&[7, 0, 128, 255]))
        .copy_bytes_at_business_boundary(1)
        .expect("byte conversion should be valid");

    let after = ScriptHostHotPathMetrics::snapshot();
    assert_eq!(text, "copy-at-boundary");
    assert_eq!(bytes, vec![7, 0, 128, 255]);
    assert!(after.guest_string_copy_bytes >= before.guest_string_copy_bytes + text.len() as u64);
    assert!(after.guest_byte_copy_bytes >= before.guest_byte_copy_bytes + bytes.len() as u64);
}
