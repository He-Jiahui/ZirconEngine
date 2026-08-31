use super::super::{SchemaId, SchemaIdError, MAX_SCHEMA_ID_BYTES};

#[test]
fn schema_id_deserializes_to_owned_storage_and_round_trips() {
    let schema_id = {
        let bytes = br#""zircon.tests.owned-schema""#.to_vec();
        serde_json::from_slice::<SchemaId>(&bytes).unwrap()
    };

    assert_eq!(schema_id.as_str(), "zircon.tests.owned-schema");
    assert_eq!(
        serde_json::to_string(&schema_id).unwrap(),
        r#""zircon.tests.owned-schema""#
    );
}

#[test]
fn schema_id_accepts_the_portable_namespace_grammar_at_the_length_limit() {
    const STATIC_ID: SchemaId = SchemaId::new("zircon.tests.static-schema-v1");
    let boundary = format!("a.{}", "b".repeat(MAX_SCHEMA_ID_BYTES - 2));

    assert_eq!(STATIC_ID.as_str(), "zircon.tests.static-schema-v1");
    assert_eq!(boundary.len(), MAX_SCHEMA_ID_BYTES);
    assert_eq!(
        SchemaId::try_from(boundary.clone()).unwrap().as_str(),
        boundary
    );
}

#[test]
fn schema_id_rejects_non_portable_or_ambiguous_wire_names() {
    let cases = [
        ("", SchemaIdError::Empty),
        ("zircon", SchemaIdError::MissingNamespace),
        (".zircon", SchemaIdError::EmptySegment { index: 0 }),
        ("zircon.", SchemaIdError::EmptySegment { index: 7 }),
        ("zircon..scene", SchemaIdError::EmptySegment { index: 7 }),
        (
            "Zircon.scene",
            SchemaIdError::InvalidSegmentStart {
                index: 0,
                found: 'Z',
            },
        ),
        (
            "zircon.-scene",
            SchemaIdError::InvalidSegmentStart {
                index: 7,
                found: '-',
            },
        ),
        (
            "zircon.scene-",
            SchemaIdError::InvalidSegmentEnd {
                index: 12,
                found: '-',
            },
        ),
        (
            "zircon.scene_name",
            SchemaIdError::InvalidCharacter {
                index: 12,
                found: '_',
            },
        ),
        ("zircon.场景", SchemaIdError::NonAscii { index: 7 }),
    ];

    for (value, expected) in cases {
        assert_eq!(SchemaId::try_from(value), Err(expected), "value={value:?}");
        assert!(serde_json::from_str::<SchemaId>(&format!("{value:?}")).is_err());
    }
}

#[test]
fn schema_id_rejects_wire_names_above_the_length_limit() {
    let value = format!("a.{}", "b".repeat(MAX_SCHEMA_ID_BYTES - 1));

    assert_eq!(value.len(), MAX_SCHEMA_ID_BYTES + 1);
    assert_eq!(
        SchemaId::try_from(value),
        Err(SchemaIdError::TooLong {
            max: MAX_SCHEMA_ID_BYTES,
            found: MAX_SCHEMA_ID_BYTES + 1,
        })
    );
}
