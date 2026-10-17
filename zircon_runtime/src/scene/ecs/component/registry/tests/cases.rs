use super::*;

// 测试把 Rust table、SparseSet、dynamic 与 transferred descriptor 的注册契约放在一起核对。
struct TableValue;

zircon_runtime_interface::impl_retained_owner_v1!(TableValue {});
impl Component for TableValue {}

struct SparseValue;

zircon_runtime_interface::impl_retained_owner_v1!(SparseValue {});
impl Component for SparseValue {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

#[test]
fn rust_table_components_receive_their_registered_dense_column_layout() {
    let mut registry = ComponentRegistry::default();
    let table_id = registry.component_id::<TableValue>();
    let sparse_id = registry.component_id::<SparseValue>();
    let dynamic_id = registry.dynamic_component_id("plugin.example.dynamic");

    assert!(registry
        .table_column_layout(table_id)
        .is_some_and(TableColumnLayout::matches::<TableValue>));
    assert!(registry.table_column_layout(sparse_id).is_none());
    assert!(registry.table_column_layout(dynamic_id).is_none());
}

#[test]
fn table_column_layout_batches_preserve_the_signature_component_order() {
    let mut registry = ComponentRegistry::default();
    let first = registry.component_id::<TableValue>();
    let second = registry.component_id::<AnotherTableValue>();
    let sparse = registry.component_id::<SparseValue>();

    let layouts = registry
        .table_column_layouts_for_ids(&[second, first])
        .expect("registered table components must have layouts");

    assert_eq!(
        layouts
            .iter()
            .map(|(component_id, _)| *component_id)
            .collect::<Vec<_>>(),
        vec![second, first]
    );
    assert!(registry
        .table_column_layouts_for_ids(&[first, sparse])
        .is_none());
}

#[test]
fn transferred_rust_table_descriptor_imports_its_column_layout_once() {
    let mut source = ComponentRegistry::default();
    let source_id = source.component_id::<TableValue>();
    let transfer = source
        .transferred_descriptor(source_id)
        .expect("registered source descriptor should transfer");

    let mut target = ComponentRegistry::default();
    let mut imports = target.begin_transferred_descriptor_imports();
    let imported = target
        .preflight_transferred_descriptor_import(&mut imports, &transfer)
        .expect("target should preflight a compatible table descriptor");
    let resolved_again = target
        .preflight_transferred_descriptor_import(&mut imports, &transfer)
        .expect("target should reuse its local descriptor identity during preflight");
    target.publish_preflighted_transferred_descriptor_imports(imports);

    assert_eq!(imported, resolved_again);
    assert_eq!(
        target.rust_type_for_id(imported),
        Some((TypeId::of::<TableValue>(), type_name::<TableValue>()))
    );
    assert!(target
        .table_column_layout(imported)
        .is_some_and(TableColumnLayout::matches::<TableValue>));
}

#[test]
fn dynamic_descriptor_preflight_reserves_one_unpublished_target_local_id() {
    let mut registry = ComponentRegistry::default();
    let mut imports = registry.begin_transferred_descriptor_imports();

    let first =
        registry.preflight_dynamic_descriptor_import(&mut imports, "weather.Component.CloudLayer");
    let second =
        registry.preflight_dynamic_descriptor_import(&mut imports, "weather.Component.CloudLayer");

    assert_eq!(first, second);
    assert!(registry
        .registered_dynamic_component_id("weather.Component.CloudLayer")
        .is_none());
    assert_eq!(imports.imported_descriptor_count(), 1);

    registry.publish_preflighted_transferred_descriptor_imports(imports);
    assert_eq!(
        registry.registered_dynamic_component_id("weather.Component.CloudLayer"),
        Some(first)
    );
}

#[test]
fn transferred_descriptor_preflight_defers_import_until_publication() {
    let mut source = ComponentRegistry::default();
    let source_id = source.component_id::<TableValue>();
    let transfer = source
        .transferred_descriptor(source_id)
        .expect("registered source descriptor should transfer");

    let mut target = ComponentRegistry::default();
    target.component_id::<SparseValue>();
    let base_descriptor_count = target.descriptors().len();
    let mut imports = target.begin_transferred_descriptor_imports();

    let imported = target
        .preflight_transferred_descriptor_import(&mut imports, &transfer)
        .expect("compatible table descriptor should preflight");
    let reused = target
        .preflight_transferred_descriptor_import(&mut imports, &transfer)
        .expect("repeated descriptor should resolve inside the same preflight");

    assert_eq!(imported, reused);
    assert_eq!(imports.imported_descriptor_count(), 1);
    assert_eq!(imports.reused_descriptor_resolve_count(), 1);
    assert_eq!(target.descriptors().len(), base_descriptor_count);
    assert!(target.table_column_layout(imported).is_none());

    target.publish_preflighted_transferred_descriptor_imports(imports);

    assert_eq!(target.descriptors().len(), base_descriptor_count + 1);
    assert!(target
        .table_column_layout(imported)
        .is_some_and(TableColumnLayout::matches::<TableValue>));
}

#[test]
fn transferred_descriptor_preflight_rejects_conflicts_without_mutating_the_base() {
    let mut target = ComponentRegistry::default();
    target.dynamic_component_id("plugin.example.Probe");
    let base_descriptor_count = target.descriptors().len();
    let conflicting = TransferredComponentDescriptor {
        descriptor: ComponentDescriptor {
            id: ComponentId::new(99),
            type_name: "incompatible".to_string(),
            storage_type: StorageType::SparseSet,
            source: ComponentDescriptorSource::DynamicPlugin {
                component_type_id: "plugin.example.Probe".to_string(),
            },
        },
        table_column_layout: None,
        retention_provider: None,
    };
    let mut imports = target.begin_transferred_descriptor_imports();

    assert!(target
        .preflight_transferred_descriptor_import(&mut imports, &conflicting)
        .is_none());
    assert_eq!(imports.imported_descriptor_count(), 0);
    assert_eq!(target.descriptors().len(), base_descriptor_count);
}

struct AnotherTableValue;

zircon_runtime_interface::impl_retained_owner_v1!(AnotherTableValue {});
impl Component for AnotherTableValue {}

struct RetainedTransferValueV1(Vec<u8>);
zircon_runtime_interface::impl_retained_owner_v1!(RetainedTransferValueV1 { 0 });
impl Component for RetainedTransferValueV1 {}

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
