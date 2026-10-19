use zircon_runtime_interface::retained_memory::{
    RetainedBoundBuilderV1, RETAINED_PROVIDER_VERSION_V1,
};

#[test]
fn transferred_component_retention_provider_keeps_type_and_cost_version() {
    use zircon_runtime_interface::retained_memory::{
        verified_std_model_v1, RetainedBoundBuilderV1,
    };
    let model = verified_std_model_v1().unwrap();
    let mut source = ComponentRegistry::default();
    let source_id = source.component_id::<RetainedTransferValueV1>();
    let source_provider = source.retention_provider_v1(source_id).unwrap();
    let transfer = source.transferred_descriptor(source_id).unwrap();
    let mut target = ComponentRegistry::default();
    let mut imports = target.begin_transferred_descriptor_imports();
    let target_id = target
        .preflight_transferred_descriptor_import(&mut imports, &transfer)
        .unwrap();
    target.publish_preflighted_transferred_descriptor_imports(imports);
    let target_provider = target.retention_provider_v1(target_id).unwrap();

    assert_eq!(target_provider, source_provider);
    assert_eq!(
        target_provider.type_id,
        TypeId::of::<RetainedTransferValueV1>()
    );
    assert_eq!(target_provider.version, RETAINED_PROVIDER_VERSION_V1);
    let payload = RetainedTransferValueV1(Vec::with_capacity(8192));
    let mut source_bound = RetainedBoundBuilderV1::new(model);
    source_provider
        .measure(&payload, &mut source_bound)
        .unwrap();
    let mut target_bound = RetainedBoundBuilderV1::new(model);
    target_provider
        .measure(&payload, &mut target_bound)
        .unwrap();
    assert_eq!(target_bound.bytes(), source_bound.bytes());
    assert!(target_bound.bytes().get() >= 8192);
    assert!(target_provider
        .measure(&SparseValue, &mut RetainedBoundBuilderV1::new(model))
        .is_err());
    let mut wrong_version = target_provider;
    wrong_version.version += 1;
    assert!(wrong_version
        .measure(&payload, &mut RetainedBoundBuilderV1::new(model))
        .is_err());
}

#[test]
fn dynamic_descriptor_retained_capability_matches_pending_and_published_transfers() {
    let type_path = "weather.Component.RetainedPresence";
    let provider = ComponentRetentionProviderV1::of::<SparseValue>();
    let mut source = ComponentRegistry::default();
    let mut source_imports = source.begin_transferred_descriptor_imports();
    let source_id = source
        .preflight_dynamic_descriptor_import(&mut source_imports, type_path, provider)
        .unwrap();
    assert!(source.registered_dynamic_component_id(type_path).is_none());
    assert!(source.retention_provider_v1(source_id).is_none());
    source.publish_preflighted_transferred_descriptor_imports(source_imports);
    let transfer = source.transferred_descriptor(source_id).unwrap();
    assert_eq!(transfer.retention_provider, Some(provider));

    let mut target = ComponentRegistry::default();
    target.component_id::<TableValue>();
    let mut imports = target.begin_transferred_descriptor_imports();
    let target_id = target
        .preflight_dynamic_descriptor_import(&mut imports, type_path, provider)
        .unwrap();
    assert_ne!(target_id, source_id);
    assert_eq!(
        target.preflight_transferred_descriptor_import(&mut imports, &transfer),
        Some(target_id)
    );
    assert!(target.registered_dynamic_component_id(type_path).is_none());
    assert!(target.retention_provider_v1(target_id).is_none());
    target.publish_preflighted_transferred_descriptor_imports(imports);
    assert_eq!(target.retention_provider_v1(target_id), Some(provider));

    let mut reused = target.begin_transferred_descriptor_imports();
    assert_eq!(
        target.preflight_dynamic_descriptor_import(&mut reused, type_path, provider),
        Some(target_id)
    );
    assert_eq!(
        target.preflight_transferred_descriptor_import(&mut reused, &transfer),
        Some(target_id)
    );
    assert_eq!(reused.imported_descriptor_count(), 0);
    let mut bound = RetainedBoundBuilderV1::new(
        zircon_runtime_interface::retained_memory::verified_std_model_v1().unwrap(),
    );
    target
        .retention_provider_v1(target_id)
        .unwrap()
        .measure(&SparseValue, &mut bound)
        .unwrap();
}

struct AnotherSparseValue;
zircon_runtime_interface::impl_retained_owner_v1!(AnotherSparseValue {});
impl Component for AnotherSparseValue {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

#[test]
fn dynamic_descriptor_retained_capability_rejects_pending_and_published_owner_conflicts() {
    let type_path = "weather.Component.RetainedPresenceConflict";
    let provider = ComponentRetentionProviderV1::of::<SparseValue>();
    let different = ComponentRetentionProviderV1::of::<AnotherSparseValue>();
    let mut registry = ComponentRegistry::default();
    let mut imports = registry.begin_transferred_descriptor_imports();
    let id = registry
        .preflight_dynamic_descriptor_import(&mut imports, type_path, provider)
        .unwrap();
    assert!(registry
        .preflight_dynamic_descriptor_import(&mut imports, type_path, different)
        .is_none());
    assert_eq!(imports.imported_descriptor_count(), 1);
    assert!(registry
        .registered_dynamic_component_id(type_path)
        .is_none());
    registry.publish_preflighted_transferred_descriptor_imports(imports);
    assert_eq!(registry.retention_provider_v1(id), Some(provider));

    let mut incompatible = registry.transferred_descriptor(id).unwrap();
    incompatible.retention_provider = Some(different);
    let mut rejected = registry.begin_transferred_descriptor_imports();
    assert!(registry
        .preflight_dynamic_descriptor_import(&mut rejected, type_path, different)
        .is_none());
    assert!(registry
        .preflight_transferred_descriptor_import(&mut rejected, &incompatible)
        .is_none());
    let mut wrong_version = provider;
    wrong_version.version += 1;
    assert!(registry
        .preflight_dynamic_descriptor_import(&mut rejected, type_path, wrong_version)
        .is_none());
    assert_eq!(rejected.imported_descriptor_count(), 0);
    assert_eq!(registry.retention_provider_v1(id), Some(provider));

    let mut undeclared = ComponentRegistry::default();
    let unbound_id = undeclared.dynamic_component_id(type_path);
    let mut unbound = undeclared.begin_transferred_descriptor_imports();
    assert!(undeclared
        .preflight_dynamic_descriptor_import(&mut unbound, type_path, provider)
        .is_none());
    assert!(undeclared.retention_provider_v1(unbound_id).is_none());
    assert_eq!(unbound.imported_descriptor_count(), 0);
}
