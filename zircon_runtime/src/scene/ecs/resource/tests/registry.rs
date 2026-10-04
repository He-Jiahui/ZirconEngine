use super::*;

struct TypedResource;
impl Resource for TypedResource {}

#[test]
fn external_resource_ids_are_stable_and_do_not_alias_rust_resources() {
    let mut registry = ResourceRegistry::default();
    let typed = registry.resource_id::<TypedResource>();
    let external = registry.external_resource_id("physics.solver");

    assert_ne!(typed, external);
    assert_eq!(registry.external_resource_id("physics.solver"), external);
    assert_eq!(
        registry.registered_external_resource_id("physics.solver"),
        Some(external)
    );
    assert!(matches!(
        &registry.descriptor(external).unwrap().source,
        ResourceDescriptorSource::ExternalNative { stable_id }
            if stable_id == "physics.solver"
    ));
}
