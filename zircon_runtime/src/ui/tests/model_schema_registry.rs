use crate::ui::binding::{UiModelSchemaRegistrationError, UiModelSchemaRegistry};
use zircon_runtime_interface::ui::{
    binding::{
        UiModelFieldAccess, UiModelFieldId, UiModelFieldSchema, UiModelProviderId,
        UiModelProviderSchema, UiModelProviderVersion, UiModelSchema, UiModelSchemaId,
        UiModelSchemaVersion,
    },
    component::UiValueKind,
};

#[test]
fn model_schema_registry_resolves_exact_provider_schema_version_and_field_access() {
    let schema = model_schema(3, 2);
    let provider = UiModelProviderSchema::new(
        UiModelProviderId::try_new("editor.selection.current").unwrap(),
        UiModelProviderVersion::try_new(5).unwrap(),
        schema.key().clone(),
    );
    let provider_key = provider.key().clone();
    let field_id = UiModelFieldId::try_new("inventory.field_1").unwrap();
    let mut registry = UiModelSchemaRegistry::new();

    assert!(registry.register_schema(schema).unwrap());
    assert!(registry.register_provider(provider).unwrap());
    assert_eq!(registry.revision(), 2);

    let field = registry.resolve_field(&provider_key, &field_id).unwrap();
    assert_eq!(field.value_kind, UiValueKind::Float);
    assert_eq!(field.access, UiModelFieldAccess::ReadWrite);
}

#[test]
fn model_schema_registry_rejects_missing_schema_duplicate_fields_and_identity_collisions() {
    let schema = model_schema(1, 1);
    let provider = UiModelProviderSchema::new(
        UiModelProviderId::try_new("editor.selection.current").unwrap(),
        UiModelProviderVersion::try_new(1).unwrap(),
        schema.key().clone(),
    );
    let mut registry = UiModelSchemaRegistry::new();

    assert!(matches!(
        registry.register_provider(provider.clone()),
        Err(UiModelSchemaRegistrationError::MissingModelSchema { .. })
    ));
    assert!(registry.register_schema(schema.clone()).unwrap());
    assert!(!registry.register_schema(schema.clone()).unwrap());

    let mut conflicting_schema = schema.clone();
    conflicting_schema.fields[0].access = UiModelFieldAccess::ReadOnly;
    assert!(matches!(
        registry.register_schema(conflicting_schema),
        Err(UiModelSchemaRegistrationError::SchemaIdentityCollision { .. })
    ));

    let duplicate_field = schema.fields[0].clone();
    let duplicate_schema = UiModelSchema::new(
        UiModelSchemaId::try_new("editor.duplicate").unwrap(),
        UiModelSchemaVersion::try_new(1).unwrap(),
        vec![duplicate_field.clone(), duplicate_field],
    );
    assert!(matches!(
        registry.register_schema(duplicate_schema),
        Err(UiModelSchemaRegistrationError::DuplicateField { .. })
    ));

    assert!(registry.register_provider(provider.clone()).unwrap());
    assert!(!registry.register_provider(provider.clone()).unwrap());
    let schema_v2 = model_schema(2, 1);
    assert!(registry.register_schema(schema_v2.clone()).unwrap());
    let conflicting_provider = UiModelProviderSchema::new(
        provider.id().clone(),
        provider.version(),
        schema_v2.key().clone(),
    );
    assert!(matches!(
        registry.register_provider(conflicting_provider),
        Err(UiModelSchemaRegistrationError::ProviderIdentityCollision { .. })
    ));
}

#[test]
fn model_schema_registry_rejects_empty_schemas_and_unknown_resolution_keys() {
    let empty_schema = UiModelSchema::new(
        UiModelSchemaId::try_new("editor.empty").unwrap(),
        UiModelSchemaVersion::try_new(1).unwrap(),
        Vec::new(),
    );
    let unknown_provider = UiModelProviderSchema::new(
        UiModelProviderId::try_new("editor.unknown").unwrap(),
        UiModelProviderVersion::try_new(1).unwrap(),
        model_schema(1, 1).key().clone(),
    );
    let unknown_field = UiModelFieldId::try_new("missing.field").unwrap();
    let mut registry = UiModelSchemaRegistry::new();

    assert!(matches!(
        registry.register_schema(empty_schema),
        Err(UiModelSchemaRegistrationError::EmptySchema { .. })
    ));
    assert!(matches!(
        registry.resolve_field(unknown_provider.key(), &unknown_field),
        Err(UiModelSchemaRegistrationError::UnknownProvider { .. })
    ));
}

#[test]
fn model_schema_registry_keeps_versions_and_large_field_sets_deterministic() {
    let mut registry = UiModelSchemaRegistry::new();
    let schema_v1 = model_schema(1, 1_024);
    let schema_v2 = model_schema(2, 1_024);
    let last_field = UiModelFieldId::try_new("inventory.field_1023").unwrap();

    assert!(registry.register_schema(schema_v2.clone()).unwrap());
    assert!(registry.register_schema(schema_v1.clone()).unwrap());

    assert_eq!(
        registry.schema_keys().cloned().collect::<Vec<_>>(),
        vec![schema_v1.key().clone(), schema_v2.key().clone()]
    );
    assert_eq!(
        registry
            .schema(schema_v2.key())
            .unwrap()
            .field(&last_field)
            .unwrap()
            .id,
        last_field
    );
}

fn model_schema(version: u64, field_count: usize) -> UiModelSchema {
    UiModelSchema::new(
        UiModelSchemaId::try_new("editor.selection").unwrap(),
        UiModelSchemaVersion::try_new(version).unwrap(),
        (0..field_count)
            .map(|index| {
                UiModelFieldSchema::new(
                    UiModelFieldId::try_new(format!("inventory.field_{index}")).unwrap(),
                    UiValueKind::Float,
                    UiModelFieldAccess::ReadWrite,
                )
            })
            .collect(),
    )
}
