use super::*;

#[test]
fn positional_formats_preserve_historical_v1_integer_and_string_bytes() {
    let value = (true, 251_u32, -3_i32, "ok".to_owned());
    let fixed = [
        1, 251, 0, 0, 0, 253, 255, 255, 255, 2, 0, 0, 0, 0, 0, 0, 0, b'o', b'k',
    ];
    let variable = [1, 251, 251, 0, 5, 2, b'o', b'k'];
    for (codec, historical) in [
        (PositionalEncoding::FixedLittleEndian, fixed.as_slice()),
        (
            PositionalEncoding::VariableLittleEndian,
            variable.as_slice(),
        ),
    ] {
        assert_eq!(codec.encode(&value).unwrap(), historical);
        assert_eq!(
            codec
                .decode::<(bool, u32, i32, String)>(historical)
                .unwrap(),
            value
        );
        let mut streamed = Vec::new();
        codec
            .encode_into(&mut streamed, &value, historical.len() as u64)
            .unwrap();
        assert_eq!(streamed, historical);
        assert_eq!(
            codec
                .decode_from::<(bool, u32, i32, String)>(historical, historical.len() as u64)
                .unwrap(),
            value
        );
    }
}

#[test]
fn positional_caps_and_trailing_bytes_are_enforced() {
    let codec = PositionalEncoding::VariableLittleEndian;
    assert!(matches!(
        codec.encode_bounded(&"long", 2),
        Err(PositionalEncodeError::SizeLimit { max: 2 })
    ));
    assert!(matches!(
        codec.decode_bounded::<u8>(&[1, 2], 1),
        Err(PositionalDecodeError::SizeLimit { max: 1 })
    ));
    assert!(matches!(
        codec.decode::<u8>(&[1, 2]),
        Err(PositionalDecodeError::TrailingBytes { remaining: 1 })
    ));
    assert_eq!(
        codec.decode_prefix_bounded::<u8>(&[1, 2], 2).unwrap(),
        (1, 1)
    );
}

#[test]
fn positional_variable_integer_budget_uses_wire_size_for_small_wide_values() {
    let codec = PositionalEncoding::VariableLittleEndian;
    let value = (1_u128, -1_i128, vec![0_u64, 1, 2]);
    let bytes = codec.encode(&value).unwrap();
    assert_eq!(bytes.len(), 6);
    assert_eq!(
        codec.decode::<(u128, i128, Vec<u64>)>(&bytes).unwrap(),
        value
    );
    assert_eq!(
        codec
            .decode_from::<(u128, i128, Vec<u64>)>(bytes.as_slice(), bytes.len() as u64)
            .unwrap(),
        value
    );
}

#[test]
fn positional_untrusted_string_lengths_do_not_allocate_unbounded_memory() {
    let declared = u64::MAX.to_le_bytes();
    let codec = PositionalEncoding::FixedLittleEndian;
    assert!(codec.decode::<String>(&declared).is_err());
    assert!(codec.decode_from::<String>(declared.as_slice(), 8).is_err());
}

#[test]
fn positional_decoder_keeps_the_existing_large_artifact_size_range() {
    // An incomplete payload must fail on input length, rather than rejecting
    // a supported asset length before reading its bytes. No large allocation.
    let declared = (300_u64 * 1024 * 1024).to_le_bytes();
    assert!(matches!(
        PositionalEncoding::FixedLittleEndian
            .decode_prefix_bounded::<String>(&declared, MAX_DECODE_ALLOCATION_BYTES as u64),
        Err(PositionalDecodeError::Codec(
            bincode_next::error::DecodeError::UnexpectedEnd { .. }
        ))
    ));
}

#[test]
fn positional_stream_budget_rejects_container_claims_before_reading_payload() {
    struct HeaderOnly(std::io::Cursor<[u8; 8]>);
    impl Read for HeaderOnly {
        fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            assert!(
                self.0.position() < 8,
                "oversized payload must not be requested"
            );
            self.0.read(bytes)
        }
    }
    let declared = 4096_u64.to_le_bytes();
    let codec = PositionalEncoding::FixedLittleEndian;
    assert!(matches!(
        codec.decode_from::<String>(HeaderOnly(std::io::Cursor::new(declared)), 64),
        Err(PositionalDecodeError::Codec(
            bincode_next::error::DecodeError::LimitExceeded
        ))
    ));
    assert!(matches!(
        codec.decode_prefix_bounded::<String>(&declared, 64),
        Err(PositionalDecodeError::Codec(
            bincode_next::error::DecodeError::LimitExceeded
        ))
    ));
}

#[test]
fn positional_variable_stream_rejects_oversized_byte_buffers_before_reading() {
    let codec = PositionalEncoding::VariableLittleEndian;
    let declared = codec.encode(&4096_u64).unwrap();
    assert!(matches!(
        codec.decode_from::<String>(declared.as_slice(), 64),
        Err(PositionalDecodeError::Codec(
            bincode_next::error::DecodeError::LimitExceeded
        ))
    ));
}
