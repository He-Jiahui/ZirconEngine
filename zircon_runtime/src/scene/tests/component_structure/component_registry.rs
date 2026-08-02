#[test]
fn component_registry_rust_type_reverse_lookup_uses_descriptor_source() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let registry_source =
        std::fs::read_to_string(manifest_root.join("src/scene/ecs/component/registry.rs")).unwrap();
    let component_id_body = registry_source
        .split("pub fn component_id<T>")
        .nth(1)
        .and_then(|text| text.split("pub fn dynamic_component_id").next())
        .expect("component_id body should exist");
    let rust_type_lookup_body = registry_source
        .split("pub(crate) fn rust_type_for_id")
        .nth(1)
        .and_then(|text| text.split("pub fn descriptors").next())
        .expect("rust_type_for_id body should exist");

    assert!(registry_source.contains("RustType { type_id: TypeId }"));
    assert!(component_id_body.contains("let type_id = TypeId::of::<T>();"));
    assert!(component_id_body.contains("self.rust_ids_by_type_id.get(&type_id).copied()"));
    assert!(component_id_body.contains("ComponentDescriptorSource::RustType { type_id }"));
    assert!(component_id_body.contains("self.rust_ids_by_type_id.insert(type_id, id);"));
    assert!(rust_type_lookup_body.contains("let descriptor = self.descriptor(id)?;"));
    assert!(rust_type_lookup_body.contains("match &descriptor.source"));
    assert!(rust_type_lookup_body.contains("Some((*type_id, descriptor.type_name.as_str()))"));
    assert!(!registry_source.contains("pub enum ComponentKey"));
    assert!(!registry_source.contains("ids_by_key"));
    assert!(!rust_type_lookup_body.contains("self.rust_ids_by_type_id.iter().find_map"));
    assert!(!rust_type_lookup_body.contains("self.descriptors[id.index()].type_name.clone()"));
}

#[test]
fn component_registry_dynamic_lookup_uses_borrowed_type_id_map() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let registry_source =
        std::fs::read_to_string(manifest_root.join("src/scene/ecs/component/registry.rs")).unwrap();
    let dynamic_id_body = registry_source
        .split("pub fn dynamic_component_id")
        .nth(1)
        .and_then(|text| text.split("pub fn registered_component_id").next())
        .expect("dynamic_component_id body should exist");
    let registered_dynamic_body = registry_source
        .split("pub fn registered_dynamic_component_id")
        .nth(1)
        .and_then(|text| text.split("pub fn descriptor").next())
        .expect("registered_dynamic_component_id body should exist");

    assert!(registry_source.contains("dynamic_ids_by_type_id: HashMap<String, ComponentId>"));
    assert!(
        dynamic_id_body.contains("self.dynamic_ids_by_type_id.get(component_type_id).copied()")
    );
    assert!(dynamic_id_body.contains("self.dynamic_ids_by_type_id"));
    assert!(dynamic_id_body.contains(".insert(component_type_id.to_string(), id);"));
    assert!(
        registered_dynamic_body
            .contains("self.dynamic_ids_by_type_id.get(component_type_id).copied()")
    );
    assert!(!registry_source.contains("pub enum ComponentKey"));
    assert!(!registry_source.contains("ids_by_key"));
    assert!(!registered_dynamic_body.contains("ComponentKey::Dynamic"));
    assert!(!registered_dynamic_body.contains("component_type_id.to_string()"));
}
