use crate::ui::binding::{UiModelSchemaRegistrationError, UiModelSchemaRegistry};
use zircon_runtime_interface::ui::{
    binding::{
        UiModelContextLayer, UiModelContextPatch, UiModelFieldAccess, UiModelFieldId,
        UiModelFieldSchema, UiModelProviderId, UiModelProviderKey, UiModelProviderSchema,
        UiModelProviderVersion, UiModelSchema, UiModelSchemaId, UiModelSchemaVersion,
        UiResolvedModelContext,
    },
    component::UiValueKind,
};

#[test]
fn registry_resolves_layered_context_with_inheritance_override_and_clear() {
    let registry = registry_with_providers(&[
        ("editor.scene", 1),
        ("editor.inspector", 2),
        ("editor.inspector.preview", 5),
        ("editor.outliner.row", 3),
        ("editor.asset.item", 4),
    ]);
    let root = registry
        .resolve_model_context(
            None,
            &UiModelContextPatch::default()
                .with_binding(UiModelContextLayer::Surface, provider("editor.scene", 1))
                .with_binding(
                    UiModelContextLayer::Component,
                    provider("editor.inspector", 2),
                )
                .with_binding(UiModelContextLayer::Row, provider("editor.outliner.row", 3))
                .with_binding(UiModelContextLayer::Item, provider("editor.asset.item", 4)),
        )
        .unwrap();

    let child = registry
        .resolve_model_context(
            Some(&root),
            &UiModelContextPatch::default()
                .with_binding(
                    UiModelContextLayer::Component,
                    provider("editor.inspector.preview", 5),
                )
                .with_clear(UiModelContextLayer::Row),
        )
        .unwrap();

    assert_eq!(
        child.provider(UiModelContextLayer::Surface),
        Some(&provider("editor.scene", 1))
    );
    assert_eq!(
        child.provider(UiModelContextLayer::Component),
        Some(&provider("editor.inspector.preview", 5))
    );
    assert_eq!(child.provider(UiModelContextLayer::Row), None);
    assert_eq!(
        child.provider(UiModelContextLayer::Item),
        Some(&provider("editor.asset.item", 4))
    );
    assert_eq!(registry.revision(), 6);
}

#[test]
fn registry_rejects_unknown_or_wrong_version_context_providers() {
    let registry = registry_with_providers(&[("editor.scene", 1)]);
    let unknown = provider("editor.scene", 2);

    let error = registry
        .resolve_model_context(
            None,
            &UiModelContextPatch::default()
                .with_binding(UiModelContextLayer::Surface, unknown.clone()),
        )
        .unwrap_err();

    assert_eq!(
        error,
        UiModelSchemaRegistrationError::UnknownContextProvider {
            layer: UiModelContextLayer::Surface,
            provider: unknown,
        }
    );
}

#[test]
fn registry_revalidates_inherited_context_instead_of_trusting_caller_state() {
    let registry = UiModelSchemaRegistry::new();
    let inherited = UiResolvedModelContext::resolve(
        None,
        &UiModelContextPatch::default()
            .with_binding(UiModelContextLayer::Item, provider("editor.asset.item", 1)),
    );

    assert!(matches!(
        registry.resolve_model_context(Some(&inherited), &UiModelContextPatch::default()),
        Err(UiModelSchemaRegistrationError::UnknownContextProvider {
            layer: UiModelContextLayer::Item,
            ..
        })
    ));
}

fn registry_with_providers(providers: &[(&str, u64)]) -> UiModelSchemaRegistry {
    let schema = UiModelSchema::new(
        UiModelSchemaId::try_new("editor.context").unwrap(),
        UiModelSchemaVersion::try_new(1).unwrap(),
        vec![UiModelFieldSchema::new(
            UiModelFieldId::try_new("value").unwrap(),
            UiValueKind::String,
            UiModelFieldAccess::ReadOnly,
        )],
    );
    let mut registry = UiModelSchemaRegistry::new();
    registry.register_schema(schema.clone()).unwrap();
    for (id, version) in providers {
        registry
            .register_provider(UiModelProviderSchema::new(
                UiModelProviderId::try_new(*id).unwrap(),
                UiModelProviderVersion::try_new(*version).unwrap(),
                schema.key().clone(),
            ))
            .unwrap();
    }
    registry
}

fn provider(id: &str, version: u64) -> UiModelProviderKey {
    UiModelProviderKey {
        id: UiModelProviderId::try_new(id).unwrap(),
        version: UiModelProviderVersion::try_new(version).unwrap(),
    }
}
