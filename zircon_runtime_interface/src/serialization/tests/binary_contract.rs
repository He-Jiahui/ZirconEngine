use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::super::{
    Format, MigrationChain, PayloadHeader, SchemaId, VersionedSchema,
    binary::{
        BinaryNode, BinaryValue, decode_binary_current, decode_binary_header,
        encode_binary_payload, encode_binary_value,
    },
    load_versioned, write_versioned, write_versioned_text,
};
use super::FixtureDocument;

#[test]
fn binary_writer_round_trips_the_current_payload() {
    let document = FixtureDocument {
        label: "binary-current".to_string(),
        count: 42,
    };

    let bytes = write_versioned(&document, Format::Binary).expect("binary encoding should succeed");
    let loaded = load_versioned::<FixtureDocument>(&bytes, Format::Binary)
        .expect("current binary payload should load");

    assert_eq!(loaded.value, document);
    assert_eq!(loaded.migrated_from, None);
}

#[test]
fn binary_current_payload_decodes_through_the_direct_typed_boundary() {
    let document = FixtureDocument {
        label: "binary-direct".to_string(),
        count: 73,
    };
    let bytes = write_versioned(&document, Format::Binary).expect("binary encoding should succeed");
    let (_, payload_body) = decode_binary_header(&bytes).expect("fixture header should decode");

    let decoded = decode_binary_current::<FixtureDocument>(payload_body)
        .expect("current binary payload should decode without an intermediate JSON Value");

    assert_eq!(decoded, document);
    assert!(
        !include_str!("../binary/value/direct_decode.rs").contains("serde_json::Value"),
        "the current binary decoder must not materialize a serde_json Value"
    );
}

#[test]
fn text_binary_text_conversion_preserves_the_canonical_document() {
    let document = FixtureDocument {
        label: "cross-format".to_string(),
        count: 9,
    };
    let canonical_before = write_versioned_text(&document).unwrap();
    let from_text = load_versioned::<FixtureDocument>(canonical_before.as_bytes(), Format::Text)
        .unwrap()
        .value;
    let binary = write_versioned(&from_text, Format::Binary).unwrap();
    let from_binary = load_versioned::<FixtureDocument>(&binary, Format::Binary)
        .unwrap()
        .value;
    let canonical_after = write_versioned_text(&from_binary).unwrap();

    assert_eq!(canonical_after, canonical_before);
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct JsonDomainDocument {
    value: Value,
}

impl VersionedSchema for JsonDomainDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.json-domain");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<JsonDomainDocument> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct NumericKeyBinaryDocument {
    entries: BTreeMap<u32, String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct BinarySerdeContractDocument {
    enabled: bool,
    choice: BinarySerdeContractChoice,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum BinarySerdeContractChoice {
    Unit,
    Newtype(bool),
    Struct { enabled: bool },
}

impl VersionedSchema for NumericKeyBinaryDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.binary-numeric-key");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<NumericKeyBinaryDocument> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

impl VersionedSchema for BinarySerdeContractDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.binary-serde-contract");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<BinarySerdeContractDocument> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

#[test]
fn binary_current_direct_decode_covers_bool_and_enum_variants() {
    let documents = [
        BinarySerdeContractDocument {
            enabled: true,
            choice: BinarySerdeContractChoice::Unit,
        },
        BinarySerdeContractDocument {
            enabled: false,
            choice: BinarySerdeContractChoice::Newtype(true),
        },
        BinarySerdeContractDocument {
            enabled: true,
            choice: BinarySerdeContractChoice::Struct { enabled: false },
        },
    ];

    for document in documents {
        let bytes =
            write_versioned(&document, Format::Binary).expect("binary encoding should succeed");
        let loaded = load_versioned::<BinarySerdeContractDocument>(&bytes, Format::Binary)
            .expect("current binary bool and enum payload should decode directly");

        assert_eq!(loaded.value, document);
        assert_eq!(loaded.migrated_from, None);
    }
}

#[test]
fn binary_current_direct_decode_preserves_numeric_object_key_semantics() {
    let document = NumericKeyBinaryDocument {
        entries: BTreeMap::from([(1, "one".to_string()), (42, "forty-two".to_string())]),
    };

    let bytes = write_versioned(&document, Format::Binary).expect("binary encoding should succeed");
    let loaded = load_versioned::<NumericKeyBinaryDocument>(&bytes, Format::Binary)
        .expect("current binary payload should decode numeric map keys");

    assert_eq!(loaded.value, document);
    assert_eq!(loaded.migrated_from, None);
}

#[test]
fn binary_value_domain_preserves_every_json_scalar_class() {
    let document = JsonDomainDocument {
        value: json!({
            "array": [null, true, false, -9223372036854775808_i64, 18446744073709551615_u64],
            "float": 0.125,
            "nested": { "text": "zircon" }
        }),
    };

    let bytes = write_versioned(&document, Format::Binary).unwrap();
    let loaded = load_versioned::<JsonDomainDocument>(&bytes, Format::Binary).unwrap();

    assert_eq!(loaded.value, document);
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct BulkDocument {
    rows: Vec<BulkRow>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct BulkRow {
    id: u32,
    parent: Option<u32>,
    label: String,
    enabled: bool,
}

impl VersionedSchema for BulkDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.binary-selection-bulk");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<BulkDocument> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

#[test]
fn configured_binary_wire_is_deterministic_and_smaller_than_canonical_text() {
    let document = BulkDocument {
        rows: (0..256)
            .map(|id| BulkRow {
                id,
                parent: id.checked_sub(1),
                label: format!("Entity {id:04}"),
                enabled: id % 3 != 0,
            })
            .collect(),
    };

    let first = write_versioned(&document, Format::Binary).unwrap();
    let second = write_versioned(&document, Format::Binary).unwrap();
    let text = write_versioned_text(&document).unwrap();

    eprintln!(
        "binary_selection_fixture rows=256 binary_bytes={} text_bytes={}",
        first.len(),
        text.len()
    );
    assert_eq!(first, second);
    assert!(
        first.len() < text.len(),
        "binary={} bytes, canonical text={} bytes",
        first.len(),
        text.len()
    );
}

#[test]
fn legacy_binary_payload_uses_the_same_value_migration_chain() {
    let header = PayloadHeader {
        schema_id: FixtureDocument::SCHEMA.clone(),
        schema_version: 0,
    };
    let bytes = encode_binary_payload(header, json!({ "name": "legacy-binary" }))
        .expect("test fixture should encode through the real wire owner");

    let loaded = load_versioned::<FixtureDocument>(&bytes, Format::Binary).unwrap();

    assert_eq!(
        loaded.value,
        FixtureDocument {
            label: "legacy-binary".to_string(),
            count: 2,
        }
    );
    assert_eq!(loaded.migrated_from, Some(0));
}

#[derive(Debug, Serialize, Deserialize)]
struct WrongSchemaDocument {
    label: String,
    count: u32,
}

impl VersionedSchema for WrongSchemaDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.wrong-binary-schema");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<WrongSchemaDocument> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

#[test]
fn binary_envelope_for_a_different_schema_is_rejected() {
    let bytes = write_versioned(
        &WrongSchemaDocument {
            label: "wrong".to_string(),
            count: 7,
        },
        Format::Binary,
    )
    .unwrap();

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Binary).unwrap_err();

    assert!(matches!(
        error,
        super::super::LoadError::SchemaMismatch { expected, found }
            if expected == "zircon.tests.fixture-document"
                && found == "zircon.tests.wrong-binary-schema"
    ));
}

#[test]
fn binary_wire_v1_golden_bytes_cover_header_and_every_node_variant() {
    const EXPECTED: &[u8] = &[
        90, 82, 80, 65, 89, 76, 68, 0, 1, 0, 31, 122, 105, 114, 99, 111, 110, 46, 116, 101, 115,
        116, 115, 46, 98, 105, 110, 97, 114, 121, 45, 119, 105, 114, 101, 45, 103, 111, 108, 100,
        101, 110, 7, 21, 7, 8, 8, 4, 110, 117, 108, 108, 0, 8, 4, 98, 111, 111, 108, 1, 1, 8, 3,
        105, 54, 52, 2, 3, 8, 3, 117, 54, 52, 3, 251, 44, 1, 8, 3, 102, 54, 52, 4, 0, 0, 0, 0, 0,
        0, 224, 63, 8, 6, 115, 116, 114, 105, 110, 103, 5, 1, 122, 8, 5, 97, 114, 114, 97, 121, 6,
        2, 0, 1, 0, 8, 6, 111, 98, 106, 101, 99, 116, 7, 1, 8, 1, 107, 2, 2,
    ];
    let bytes = encode_binary_value(
        PayloadHeader {
            schema_id: SchemaId::new("zircon.tests.binary-wire-golden"),
            schema_version: 7,
        },
        BinaryValue::from_nodes(vec![
            BinaryNode::Object { len: 8 },
            BinaryNode::ObjectKey("null".to_string()),
            BinaryNode::Null,
            BinaryNode::ObjectKey("bool".to_string()),
            BinaryNode::Bool(true),
            BinaryNode::ObjectKey("i64".to_string()),
            BinaryNode::I64(-2),
            BinaryNode::ObjectKey("u64".to_string()),
            BinaryNode::U64(300),
            BinaryNode::ObjectKey("f64".to_string()),
            BinaryNode::F64(0.5),
            BinaryNode::ObjectKey("string".to_string()),
            BinaryNode::String("z".to_string()),
            BinaryNode::ObjectKey("array".to_string()),
            BinaryNode::Array { len: 2 },
            BinaryNode::Null,
            BinaryNode::Bool(false),
            BinaryNode::ObjectKey("object".to_string()),
            BinaryNode::Object { len: 1 },
            BinaryNode::ObjectKey("k".to_string()),
            BinaryNode::I64(1),
        ]),
    )
    .unwrap();

    assert_eq!(bytes.as_slice(), EXPECTED);
}
