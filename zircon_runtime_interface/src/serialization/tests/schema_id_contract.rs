use super::super::SchemaId;

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
