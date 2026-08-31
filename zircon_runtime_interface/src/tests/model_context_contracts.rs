use crate::ui::binding::{
    UiModelContextLayer, UiModelContextOverride, UiModelContextPatch, UiModelProviderId,
    UiModelProviderKey, UiModelProviderVersion, UiResolvedModelContext,
};

#[test]
fn model_context_patch_round_trips_bind_and_clear_operations() {
    let patch = UiModelContextPatch::default()
        .with_binding(UiModelContextLayer::Surface, provider("editor.scene", 3))
        .with_binding(UiModelContextLayer::Row, provider("editor.outliner.row", 7))
        .with_clear(UiModelContextLayer::Item);

    let serialized = toml::to_string(&patch).unwrap();
    let decoded: UiModelContextPatch = toml::from_str(&serialized).unwrap();

    assert_eq!(decoded, patch);
    assert!(matches!(
        decoded.override_for(UiModelContextLayer::Surface),
        Some(UiModelContextOverride::Bind { provider: bound_provider })
            if bound_provider == &provider("editor.scene", 3)
    ));
    assert_eq!(decoded.override_for(UiModelContextLayer::Component), None);
    assert_eq!(
        decoded.override_for(UiModelContextLayer::Item),
        Some(&UiModelContextOverride::Clear)
    );
}

#[test]
fn model_context_resolution_is_surface_component_row_item_ordered() {
    assert_eq!(
        UiModelContextLayer::ALL,
        [
            UiModelContextLayer::Surface,
            UiModelContextLayer::Component,
            UiModelContextLayer::Row,
            UiModelContextLayer::Item,
        ]
    );

    let surface = UiResolvedModelContext::resolve(
        None,
        &UiModelContextPatch::default()
            .with_binding(UiModelContextLayer::Surface, provider("editor.scene", 1)),
    );
    let component = UiResolvedModelContext::resolve(
        Some(&surface),
        &UiModelContextPatch::default().with_binding(
            UiModelContextLayer::Component,
            provider("editor.inspector", 2),
        ),
    );
    let row = UiResolvedModelContext::resolve(
        Some(&component),
        &UiModelContextPatch::default()
            .with_binding(UiModelContextLayer::Row, provider("editor.outliner.row", 3)),
    );
    let item = UiResolvedModelContext::resolve(
        Some(&row),
        &UiModelContextPatch::default()
            .with_binding(UiModelContextLayer::Item, provider("editor.asset.item", 4)),
    );

    assert_eq!(
        UiModelContextLayer::ALL.map(|layer| item.provider(layer).cloned()),
        [
            Some(provider("editor.scene", 1)),
            Some(provider("editor.inspector", 2)),
            Some(provider("editor.outliner.row", 3)),
            Some(provider("editor.asset.item", 4)),
        ]
    );

    let overridden = UiResolvedModelContext::resolve(
        Some(&item),
        &UiModelContextPatch::default()
            .with_binding(
                UiModelContextLayer::Component,
                provider("editor.inspector.preview", 5),
            )
            .with_clear(UiModelContextLayer::Row),
    );
    assert_eq!(
        overridden.provider(UiModelContextLayer::Surface),
        item.provider(UiModelContextLayer::Surface)
    );
    assert_eq!(
        overridden.provider(UiModelContextLayer::Component),
        Some(&provider("editor.inspector.preview", 5))
    );
    assert_eq!(overridden.provider(UiModelContextLayer::Row), None);
    assert_eq!(
        overridden.provider(UiModelContextLayer::Item),
        item.provider(UiModelContextLayer::Item)
    );
}

fn provider(id: &str, version: u64) -> UiModelProviderKey {
    UiModelProviderKey {
        id: UiModelProviderId::try_new(id).unwrap(),
        version: UiModelProviderVersion::try_new(version).unwrap(),
    }
}
