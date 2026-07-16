use serde_json::json;

use serde::{Deserialize, Serialize};

use super::super::binary::{MAX_BINARY_BODY_BYTES, MAX_BINARY_STRING_BYTES};
use super::super::{
    Format, MigrationChain, SchemaId, VersionedSchema, WriteError, load_versioned, write_versioned,
    write_versioned_text,
};
use super::FixtureDocument;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct FloatProbe {
    precise: f64,
}

impl VersionedSchema for FloatProbe {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.float-probe");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<FloatProbe> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

#[derive(Serialize)]
struct OversizedBinaryProbe {
    chunks: Vec<String>,
}

impl VersionedSchema for OversizedBinaryProbe {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.oversized-binary-probe");
    const VERSION: u32 = 0;

    fn migrations() -> &'static MigrationChain<Self> {
        static MIGRATIONS: MigrationChain<OversizedBinaryProbe> = MigrationChain::new(&[]);
        &MIGRATIONS
    }
}

#[test]
fn canonical_text_writer_orders_nested_keys_and_uses_one_trailing_newline() {
    let document = FixtureDocument {
        label: "stable".to_string(),
        count: 7,
    };
    let first = write_versioned_text(&document).expect("fixture should encode");
    let second = write_versioned_text(&document).expect("fixture should encode identically");

    assert_eq!(first, second);
    assert!(first.ends_with('\n'));
    assert!(!first.ends_with("\n\n"));
    assert!(first.find("\"header\"").unwrap() < first.find("\"payload\"").unwrap());
    assert!(first.find("\"count\"").unwrap() < first.find("\"label\"").unwrap());

    let loaded = load_versioned::<FixtureDocument>(first.as_bytes(), Format::Text).unwrap();
    assert_eq!(loaded.value, document);
    assert_eq!(loaded.migrated_from, None);
}

#[test]
fn canonical_text_writer_uses_shortest_roundtrip_float_spelling() {
    let encoded = write_versioned_text(&FloatProbe { precise: 0.1 }).unwrap();
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(value["$zircon"]["payload"], json!({ "precise": 0.1 }));
    assert!(encoded.contains("\"precise\": 0.1"));
}

#[test]
fn writers_reject_non_finite_floats_instead_of_normalizing_them_to_null() {
    for precise in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for format in [Format::Text, Format::Binary] {
            let error = write_versioned(&FloatProbe { precise }, format)
                .expect_err("non-finite values must not become JSON null");

            assert!(matches!(
                error,
                WriteError::NonFiniteFloat {
                    schema_id,
                    schema_version: 0,
                    ..
                } if schema_id == "zircon.tests.float-probe"
            ));
        }
    }
}

#[test]
fn binary_writer_rejects_a_body_larger_than_the_reader_limit() {
    let document = OversizedBinaryProbe {
        chunks: (0..4)
            .map(|_| "x".repeat(MAX_BINARY_STRING_BYTES))
            .collect(),
    };

    let error = write_versioned(&document, Format::Binary)
        .expect_err("writer must not emit a body that the same wire reader rejects");

    assert!(matches!(
        error,
        WriteError::BinaryPayloadTooLarge {
            schema_id,
            schema_version: 0,
            max,
        } if schema_id == "zircon.tests.oversized-binary-probe"
            && max == MAX_BINARY_BODY_BYTES
    ));
}
