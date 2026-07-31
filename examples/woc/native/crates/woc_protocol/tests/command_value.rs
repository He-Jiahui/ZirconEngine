use woc_protocol::{
    decode_command_value, encode_command_value, CommandValue, CommandValueLimits, ProtocolError,
};

#[test]
fn command_values_round_trip_nested_data_with_canonical_object_order() {
    let unordered = CommandValue::object(vec![
        (
            "z".to_owned(),
            CommandValue::Array(vec![CommandValue::Number(1.25), CommandValue::Null]),
        ),
        ("a".to_owned(), CommandValue::Bool(true)),
    ])
    .expect("unique object keys");
    let ordered = CommandValue::object(vec![
        ("a".to_owned(), CommandValue::Bool(true)),
        (
            "z".to_owned(),
            CommandValue::Array(vec![CommandValue::Number(1.25), CommandValue::Null]),
        ),
    ])
    .expect("unique object keys");

    let unordered_bytes = encode_command_value(&unordered, CommandValueLimits::default())
        .expect("encode unordered object");
    assert_eq!(
        unordered_bytes,
        encode_command_value(&ordered, CommandValueLimits::default())
            .expect("encode ordered object")
    );
    assert_eq!(
        decode_command_value(&unordered_bytes, CommandValueLimits::default())
            .expect("decode canonical object"),
        ordered
    );
}

#[test]
fn command_value_normalizes_negative_zero_and_rejects_non_finite_numbers() {
    let encoded = encode_command_value(&CommandValue::Number(-0.0), CommandValueLimits::default())
        .expect("negative zero is JSON-equivalent to zero");
    assert_eq!(
        decode_command_value(&encoded, CommandValueLimits::default()).expect("decode zero"),
        CommandValue::Number(0.0)
    );
    assert!(matches!(
        encode_command_value(&CommandValue::Number(f64::NAN), CommandValueLimits::default()),
        Err(ProtocolError::NonFinite {
            field: "command value number",
            value,
        }) if value.is_nan()
    ));
}

#[test]
fn command_value_rejects_duplicate_keys_unknown_tags_and_trailing_bytes() {
    let duplicate_key = [6, 2, 0, 0, 0, 1, 0, 0, 0, b'a', 0, 1, 0, 0, 0, b'a', 0];
    assert_eq!(
        decode_command_value(&duplicate_key, CommandValueLimits::default()),
        Err(ProtocolError::DuplicateCommandObjectKey {
            key: "a".to_owned(),
        })
    );
    assert_eq!(
        decode_command_value(&[255], CommandValueLimits::default()),
        Err(ProtocolError::UnknownCommandValueTag(255))
    );
    assert_eq!(
        decode_command_value(&[0, 0], CommandValueLimits::default()),
        Err(ProtocolError::TrailingPayload { remaining: 1 })
    );
}

#[test]
fn command_value_limits_reject_deep_or_oversized_inputs() {
    let deeply_nested = CommandValue::Array(vec![CommandValue::Array(vec![CommandValue::Null])]);
    let limits = CommandValueLimits {
        max_value_depth: 1,
        ..CommandValueLimits::default()
    };
    assert_eq!(
        encode_command_value(&deeply_nested, limits),
        Err(ProtocolError::CollectionTooLarge {
            context: "command value depth",
            actual: 2,
            maximum: 1,
        })
    );

    let limits = CommandValueLimits {
        max_total_bytes: 4,
        ..CommandValueLimits::default()
    };
    assert_eq!(
        encode_command_value(&CommandValue::String("wolf".to_owned()), limits),
        Err(ProtocolError::CollectionTooLarge {
            context: "command value bytes",
            actual: 9,
            maximum: 4,
        })
    );
}
