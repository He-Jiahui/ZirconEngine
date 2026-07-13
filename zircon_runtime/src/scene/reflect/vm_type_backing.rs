#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmTypeBacking {
    DynamicComponent,
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use zircon_runtime_interface::reflect::{
        ReflectEditorHint, ReflectFieldInfo, ReflectObjectAddress, ReflectReadRequest,
        ReflectScriptVisibility, ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath,
        ReflectTypeRegistration, ReflectWriteRequest, ReflectedValue,
    };

    use crate::scene::{NodeKind, World};

    use super::VmTypeBacking;

    #[test]
    fn vm_type_round_trips_as_dynamic_component() {
        let mut world = World::empty();
        world
            .type_registry_mut_for_tests()
            .register_vm_type(vm_health_registration(), VmTypeBacking::DynamicComponent)
            .expect("VM component type should register in the shared registry");
        let entity = world.spawn_node(NodeKind::Empty);
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
            .reflect_read(ReflectReadRequest::new(address.clone(), "current"))
            .expect("VM component field should read through shared reflection");
        assert_eq!(read.field.value, ReflectedValue::Scalar(25.0));

        let write = world
            .reflect_write(ReflectWriteRequest::new(
                address,
                "current",
                ReflectedValue::Scalar(40.0),
            ))
            .expect("VM component field should write through shared reflection");
        assert!(write.changed);
        assert_eq!(
            world.dynamic_component(entity, "gameplay.Component.Health"),
            Some(&json!({ "current": 40.0, "maximum": 100.0 }))
        );
    }

    fn vm_health_registration() -> ReflectTypeRegistration {
        ReflectTypeRegistration::new(
            ReflectTypePath::new("gameplay.Component.Health", "Health")
                .expect("test type path should be valid"),
            "Health",
            ReflectTypeInfo::json_with_fields(vec![
                ReflectFieldInfo::new("current", "Scalar", ReflectEditorHint::Scalar),
                ReflectFieldInfo::new("maximum", "Scalar", ReflectEditorHint::Scalar)
                    .with_editable(false),
            ]),
            ReflectSerializationStrategy::Json,
        )
        .as_component()
        .with_plugin_owned(true)
        .with_plugin_id("gameplay")
        .with_script_visibility(ReflectScriptVisibility::Public)
    }
}
