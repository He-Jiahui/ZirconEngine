use super::super::{
    Format, LoadError, MigrationChain, PayloadHeader, SchemaId, VersionedSchema,
    binary::{
        BinaryNode, BinaryValue, MAX_BINARY_BODY_BYTES, MAX_BINARY_DEPTH, encode_binary_value,
    },
    load_versioned, write_versioned,
};
use super::FixtureDocument;

const BINARY_PREFIX_LEN: usize = 10;

#[test]
fn truncated_binary_prefix_is_a_typed_header_error() {
    let error = load_versioned::<FixtureDocument>(b"ZRPAYLD\0\x01", Format::Binary).unwrap_err();

    assert!(matches!(
        error,
        LoadError::BinaryHeaderTruncated {
            expected: BINARY_PREFIX_LEN,
            found: 9,
        }
    ));
}

#[test]
fn binary_magic_mismatch_is_rejected_before_payload_decode() {
    let mut bytes = current_binary_fixture();
    bytes[0] = b'X';

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Binary).unwrap_err();

    assert!(matches!(error, LoadError::BinaryMagicMismatch { .. }));
}

#[test]
fn future_binary_wire_version_is_rejected_before_payload_decode() {
    let mut bytes = current_binary_fixture();
    bytes[8..10].copy_from_slice(&2_u16.to_le_bytes());

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Binary).unwrap_err();

    assert!(matches!(
        error,
        LoadError::UnsupportedBinaryWireVersion {
            found: 2,
            supported: 1,
        }
    ));
}

#[test]
fn trailing_binary_bytes_are_rejected_instead_of_silently_ignored() {
    let mut bytes = current_binary_fixture();
    bytes.push(0);

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Binary).unwrap_err();

    assert!(matches!(error, LoadError::MalformedBinary { .. }));
}

#[test]
fn non_finite_binary_number_is_rejected_before_typed_payload_decode() {
    let bytes = encode_binary_value(
        current_header(),
        BinaryValue::from_nodes(vec![BinaryNode::F64(f64::NAN)]),
    )
    .unwrap();

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Binary).unwrap_err();

    assert!(matches!(error, LoadError::InvalidBinaryPayload { .. }));
}

#[test]
fn future_schema_header_is_rejected_before_an_invalid_value_domain_payload() {
    let bytes = encode_binary_value(
        PayloadHeader {
            schema_id: FixtureDocument::SCHEMA.clone(),
            schema_version: FixtureDocument::VERSION + 1,
        },
        BinaryValue::from_nodes(vec![BinaryNode::F64(f64::NAN)]),
    )
    .unwrap();

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Binary).unwrap_err();

    assert!(matches!(
        error,
        LoadError::FutureVersion {
            found,
            supported,
            ..
        } if found == FixtureDocument::VERSION + 1
            && supported == FixtureDocument::VERSION
    ));
}

#[test]
fn oversized_binary_body_is_rejected_before_bincode_decode() {
    let mut bytes = Vec::with_capacity(BINARY_PREFIX_LEN + MAX_BINARY_BODY_BYTES + 1);
    bytes.extend_from_slice(b"ZRPAYLD\0");
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.resize(BINARY_PREFIX_LEN + MAX_BINARY_BODY_BYTES + 1, 0);

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Binary).unwrap_err();

    assert!(matches!(
        error,
        LoadError::BinaryPayloadTooLarge {
            max,
            found,
        } if max == MAX_BINARY_BODY_BYTES && found == MAX_BINARY_BODY_BYTES + 1
    ));
}

#[test]
fn deeply_nested_binary_values_are_rejected_at_the_wire_limit() {
    let mut nodes = (0..=MAX_BINARY_DEPTH)
        .map(|_| BinaryNode::Array { len: 1 })
        .collect::<Vec<_>>();
    nodes.push(BinaryNode::Null);
    let value = BinaryValue::from_nodes(nodes);
    let bytes = encode_binary_value(current_header(), value).unwrap();

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Binary).unwrap_err();

    assert!(matches!(
        error,
        LoadError::InvalidBinaryPayload { reason }
            if reason.contains("nesting depth")
    ));
}

#[test]
fn duplicate_binary_object_keys_are_rejected_instead_of_overwritten() {
    let bytes = encode_binary_value(
        current_header(),
        BinaryValue::from_nodes(vec![
            BinaryNode::Object { len: 2 },
            BinaryNode::ObjectKey("label".to_string()),
            BinaryNode::String("first".to_string()),
            BinaryNode::ObjectKey("label".to_string()),
            BinaryNode::String("second".to_string()),
        ]),
    )
    .unwrap();

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Binary).unwrap_err();

    assert!(matches!(error, LoadError::InvalidBinaryPayload { .. }));
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct FutureBinaryDocument {
    label: String,
    count: u32,
}

impl VersionedSchema for FutureBinaryDocument {
    const SCHEMA: SchemaId = FixtureDocument::SCHEMA;
    const VERSION: u32 = 3;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<FutureBinaryDocument> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

#[test]
fn future_binary_schema_version_is_rejected_before_typed_payload_decode() {
    let bytes = write_versioned(
        &FutureBinaryDocument {
            label: "future".to_string(),
            count: 99,
        },
        Format::Binary,
    )
    .unwrap();

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Binary).unwrap_err();

    assert!(matches!(
        error,
        LoadError::FutureVersion {
            found: 3,
            supported: 2,
            ..
        }
    ));
}

fn current_binary_fixture() -> Vec<u8> {
    write_versioned(
        &FixtureDocument {
            label: "wire".to_string(),
            count: 1,
        },
        Format::Binary,
    )
    .unwrap()
}

fn current_header() -> PayloadHeader {
    PayloadHeader {
        schema_id: FixtureDocument::SCHEMA.clone(),
        schema_version: FixtureDocument::VERSION,
    }
}
