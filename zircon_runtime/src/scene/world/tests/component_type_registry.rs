use super::ComponentTypeRegistry;
use crate::core::framework::scene::ComponentTypeDescriptor;

#[test]
fn schema_generation_changes_only_for_updated_component_type() {
    let mut registry = ComponentTypeRegistry::default();
    let cloud =
        ComponentTypeDescriptor::new("weather.Component.CloudLayer", "weather", "Cloud Layer")
            .with_property("coverage", "Scalar", true);
    registry.upsert_vm_descriptor(cloud.clone()).unwrap();
    let cloud_generation = registry.schema_generation(&cloud.type_id);

    registry.upsert_vm_descriptor(cloud.clone()).unwrap();
    assert_eq!(registry.schema_generation(&cloud.type_id), cloud_generation);

    registry
        .upsert_vm_descriptor(
            ComponentTypeDescriptor::new("weather.Component.Wind", "weather", "Wind")
                .with_property("speed", "Scalar", true),
        )
        .unwrap();
    assert_eq!(registry.schema_generation(&cloud.type_id), cloud_generation);

    registry
        .upsert_vm_descriptor(cloud.with_property("density", "Scalar", false))
        .unwrap();
    assert!(registry.schema_generation("weather.Component.CloudLayer") > cloud_generation);
    assert!(registry.schema_catalog_generation() > cloud_generation);
}
