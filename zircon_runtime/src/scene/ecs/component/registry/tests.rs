use super::*;

struct TableValue;

impl Component for TableValue {}

struct SparseValue;

impl Component for SparseValue {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

#[test]
fn rust_table_components_receive_their_registered_dense_column_layout() {
    let mut registry = ComponentRegistry::default();
    let table_id = registry.component_id::<TableValue>();
    let sparse_id = registry.component_id::<SparseValue>();
    let dynamic_id = registry.dynamic_component_id("plugin.example.dynamic");

    assert!(
        registry
            .table_column_layout(table_id)
            .is_some_and(TableColumnLayout::matches::<TableValue>)
    );
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
    assert!(
        registry
            .table_column_layouts_for_ids(&[first, sparse])
            .is_none()
    );
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
    assert!(
        target
            .table_column_layout(imported)
            .is_some_and(TableColumnLayout::matches::<TableValue>)
    );
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
    assert!(
        registry
            .registered_dynamic_component_id("weather.Component.CloudLayer")
            .is_none()
    );
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
    assert!(
        target
            .table_column_layout(imported)
            .is_some_and(TableColumnLayout::matches::<TableValue>)
    );
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
    };
    let mut imports = target.begin_transferred_descriptor_imports();

    assert!(
        target
            .preflight_transferred_descriptor_import(&mut imports, &conflicting)
            .is_none()
    );
    assert_eq!(imports.imported_descriptor_count(), 0);
    assert_eq!(target.descriptors().len(), base_descriptor_count);
}

struct AnotherTableValue;

impl Component for AnotherTableValue {}
