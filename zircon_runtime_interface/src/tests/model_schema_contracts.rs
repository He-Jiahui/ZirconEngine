use crate::ui::{
    binding::{
        UiModelFieldAccess, UiModelFieldId, UiModelFieldSchema, UiModelIdentityKind,
        UiModelProviderId, UiModelProviderSchema, UiModelProviderVersion, UiModelSchema,
        UiModelSchemaId, UiModelSchemaVersion,
    },
    component::UiValueKind,
};

#[test]
fn model_schema_contract_round_trips_typed_identity_version_and_field_access() {
    let schema = UiModelSchema::new(
        UiModelSchemaId::try_new("editor.selection").unwrap(),
        UiModelSchemaVersion::try_new(3).unwrap(),
        vec![
            UiModelFieldSchema::new(
                UiModelFieldId::try_new("name").unwrap(),
                UiValueKind::String,
                UiModelFieldAccess::ReadOnly,
            ),
            UiModelFieldSchema::new(
                UiModelFieldId::try_new("transform.translation.x").unwrap(),
                UiValueKind::Float,
                UiModelFieldAccess::ReadWrite,
            ),
        ],
    );
    let provider = UiModelProviderSchema::new(
        UiModelProviderId::try_new("editor.selection.current").unwrap(),
        UiModelProviderVersion::try_new(5).unwrap(),
        schema.key().clone(),
    );

    let serialized_schema = toml::to_string(&schema).unwrap();
    let serialized_provider = toml::to_string(&provider).unwrap();
    let decoded_schema: UiModelSchema = toml::from_str(&serialized_schema).unwrap();
    let decoded_provider: UiModelProviderSchema = toml::from_str(&serialized_provider).unwrap();

    assert_eq!(decoded_schema, schema);
    assert_eq!(decoded_provider, provider);
    assert!(serialized_schema.contains("id = \"editor.selection\""));
    assert!(serialized_schema.contains("version = 3"));
    assert!(serialized_schema.contains("access = \"ReadWrite\""));
    assert!(serialized_provider.contains("id = \"editor.selection.current\""));
    assert!(serialized_provider.contains("version = 5"));
}

#[test]
fn model_schema_identity_and_versions_reject_invalid_contracts() {
    for (kind, result) in [
        (
            UiModelIdentityKind::Schema,
            UiModelSchemaId::try_new("").map(|_| ()),
        ),
        (
            UiModelIdentityKind::Field,
            UiModelFieldId::try_new("transform..x").map(|_| ()),
        ),
        (
            UiModelIdentityKind::Provider,
            UiModelProviderId::try_new("provider/current").map(|_| ()),
        ),
    ] {
        let error = result.unwrap_err();
        assert_eq!(error.kind(), kind);
    }

    assert!(UiModelSchemaVersion::try_new(0).is_err());
    assert!(UiModelProviderVersion::try_new(0).is_err());
}
