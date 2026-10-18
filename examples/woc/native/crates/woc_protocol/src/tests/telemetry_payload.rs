use super::*;

#[test]
fn roundtrips_numeric_fields_in_canonical_key_order() {
    let payload = TelemetryPayload {
        kind: "render".to_owned(),
        data: BTreeMap::from([("frame_ms".to_owned(), 16.5), ("fps".to_owned(), 60.0)]),
    };
    let encoded = payload.clone().encode().expect("payload encodes");
    assert_eq!(TelemetryPayload::decode(&encoded), Ok(payload));
}

#[test]
fn rejects_duplicate_keys_and_non_finite_values() {
    let mut duplicate = Vec::new();
    append_text(&mut duplicate, "render");
    duplicate.extend_from_slice(&2u16.to_le_bytes());
    for value in [60.0f64, 61.0] {
        append_text(&mut duplicate, "fps");
        duplicate.extend_from_slice(&value.to_le_bytes());
    }
    assert!(matches!(
        TelemetryPayload::decode(&duplicate),
        Err(ProtocolError::DuplicateCommandObjectKey { .. })
    ));

    let payload = TelemetryPayload {
        kind: "render".to_owned(),
        data: BTreeMap::from([("fps".to_owned(), f64::NAN)]),
    };
    assert!(matches!(
        payload.encode(),
        Err(ProtocolError::NonFinite { .. })
    ));
}
