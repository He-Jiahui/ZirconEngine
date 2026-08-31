use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VmTypeBacking {
    DynamicComponent,
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use zircon_runtime_interface::reflect::{
        ReflectEditorHint, ReflectError, ReflectFieldId, ReflectFieldInfo, ReflectObjectAddress,
        ReflectReadRequest, ReflectScriptVisibility, ReflectSerializationStrategy, ReflectTypeInfo,
        ReflectTypePath, ReflectTypeRegistration, ReflectWriteRequest, ReflectedValue,
    };

    use crate::scene::reflect::TypeRegistry;
    use crate::scene::{NodeKind, SceneError, World};

    use super::VmTypeBacking;

    #[test]
    fn vm_type_round_trips_as_dynamic_component() {
        let mut world = World::empty();
        world
            .register_vm_type(vm_health_registration(), VmTypeBacking::DynamicComponent)
            .expect("VM component type should register in the shared registry");
        let entity = world
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
        world
            .set_dynamic_component(
                entity,
                "gameplay.Component.Health",
                json!({ "current": 25.0, "maximum": 100.0 }),
            )
            .expect("VM-backed dynamic component should attach");
        let address = ReflectObjectAddress::component(entity, "gameplay.Component.Health")
            .expect("VM component address should be valid");

        let read = world
            .reflect_read(ReflectReadRequest::new(
                address.clone(),
                vm_health_field_id("current"),
            ))
            .expect("VM component field should read through shared reflection");
        assert_eq!(read.field.value, ReflectedValue::Scalar(25.0));

        let write = world
            .reflect_write(ReflectWriteRequest::new(
                address,
                vm_health_field_id("current"),
                ReflectedValue::Scalar(40.0),
            ))
            .expect("VM component field should write through shared reflection");
        assert!(write.changed);
        assert_eq!(
            world.dynamic_component(entity, "gameplay.Component.Health"),
            Some(&json!({ "current": 40.0, "maximum": 100.0 }))
        );
    }

    #[test]
    fn identical_vm_type_upsert_preserves_schema_catalog_generation() {
        let registration = vm_health_registration();
        let mut registry = TypeRegistry::default();
        registry
            .upsert_vm_type(registration.clone(), VmTypeBacking::DynamicComponent)
            .expect("initial VM schema should publish");
        let generation = registry.schema_catalog_generation();

        registry
            .upsert_vm_type(registration, VmTypeBacking::DynamicComponent)
            .expect("an identical VM schema should remain accepted");

        assert_eq!(registry.schema_catalog_generation(), generation);
    }

    #[test]
    fn vm_registration_rejects_duplicate_reflected_field_names() {
        let mut registration = vm_health_registration();
        registration
            .type_info
            .fields
            .push(ReflectFieldInfo::from_stable_keys(
                "tests.vm-health",
                "current-duplicate",
                "current",
                "String",
                ReflectEditorHint::String,
            ));
        let mut world = World::empty();

        let error = world
            .register_vm_type(registration, VmTypeBacking::DynamicComponent)
            .expect_err("VM field names must be unique at the shared registry boundary");

        assert!(matches!(
            error,
            SceneError::Reflect(ReflectError::InvalidFieldRegistration { ref reason, .. })
                if reason.contains("duplicate reflected field `current`")
        ));
    }

    #[test]
    fn vm_dynamic_component_write_rejects_schema_type_mismatch() {
        let mut world = World::empty();
        world
            .register_vm_type(vm_health_registration(), VmTypeBacking::DynamicComponent)
            .expect("VM component type should register through the production entry");
        let entity = world
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
        world
            .set_dynamic_component(
                entity,
                "gameplay.Component.Health",
                json!({ "current": 25.0, "maximum": 100.0 }),
            )
            .expect("VM-backed dynamic component should attach");
        let address = ReflectObjectAddress::component(entity, "gameplay.Component.Health")
            .expect("VM component address should be valid");

        let error = world
            .reflect_write(ReflectWriteRequest::new(
                address,
                vm_health_field_id("current"),
                ReflectedValue::String("not-a-scalar".to_string()),
            ))
            .expect_err("VM writes must match the authoritative reflected field schema");

        assert_eq!(
            error,
            ReflectError::TypeMismatch {
                type_path: "gameplay.Component.Health".to_string(),
                field_name: "current".to_string(),
                expected: "Scalar".to_string(),
                actual: "String".to_string(),
            }
        );
    }

    fn vm_health_registration() -> ReflectTypeRegistration {
        ReflectTypeRegistration::new(
            ReflectTypePath::new("gameplay.Component.Health", "Health")
                .expect("test type path should be valid"),
            "Health",
            ReflectTypeInfo::json_with_fields(vec![
                ReflectFieldInfo::from_stable_keys(
                    "tests.vm-health",
                    "current",
                    "current",
                    "Scalar",
                    ReflectEditorHint::Scalar,
                ),
                ReflectFieldInfo::from_stable_keys(
                    "tests.vm-health",
                    "maximum",
                    "maximum",
                    "Scalar",
                    ReflectEditorHint::Scalar,
                )
                .with_editable(false),
            ]),
            ReflectSerializationStrategy::Json,
        )
        .as_component()
        .with_plugin_id("gameplay")
        .expect("test plugin id should be valid")
        .with_script_visibility(ReflectScriptVisibility::Public)
    }

    fn vm_health_field_id(field_key: &str) -> ReflectFieldId {
        ReflectFieldId::from_stable_keys("tests.vm-health", field_key)
    }
}
