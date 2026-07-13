use serde_json::json;

use super::super::{load_versioned, write_versioned, write_versioned_text, Format, WriteError};
use super::FixtureDocument;

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
    #[derive(serde::Serialize)]
    struct FloatProbe {
        precise: f64,
    }

    impl super::super::VersionedSchema for FloatProbe {
        const SCHEMA: super::super::SchemaId =
            super::super::SchemaId::new("zircon.tests.float-probe");
        const VERSION: u32 = 0;

        fn migrations() -> &'static super::super::MigrationChain<Self> {
            static MIGRATIONS: super::super::MigrationChain<FloatProbe> =
                super::super::MigrationChain::new(&[]);
            &MIGRATIONS
        }
    }

    let encoded = write_versioned_text(&FloatProbe { precise: 0.1 }).unwrap();
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(value["$zircon"]["payload"], json!({ "precise": 0.1 }));
    assert!(encoded.contains("\"precise\": 0.1"));
}

#[test]
fn binary_writer_remains_unavailable_until_m3() {
    let error = write_versioned(
        &FixtureDocument {
            label: "binary".to_string(),
            count: 1,
        },
        Format::Binary,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        WriteError::UnsupportedFormat {
            format: Format::Binary
        }
    ));
}
