use super::*;

#[test]
fn world_reflection_routes_component_and_resource_addresses() {
    let mut world = World::empty();
    world.type_registry_mut_for_tests().clear();
    let entity = world.spawn_node(NodeKind::Mesh);

    world
        .type_registry_mut_for_tests()
        .register(RuntimeTypeRegistration {
            registration: typed_registration("plugin_a::ProbeComponent", "ProbeComponent")
                .as_component(),
            component: Some(dummy_component_adapter()),
            resource: None,
        })
        .expect("component registration should be accepted");
    world
        .type_registry_mut_for_tests()
        .register(RuntimeTypeRegistration {
            registration: typed_registration("plugin_a::ProbeResource", "ProbeResource")
                .as_resource()
                .with_remote_visible(true),
            component: None,
            resource: Some(dummy_resource_adapter()),
        })
        .expect("resource registration should be accepted");
    world
        .type_registry_mut_for_tests()
        .register(RuntimeTypeRegistration::metadata(
            typed_registration("plugin_a::PluginHidden", "PluginHidden")
                .as_component()
                .with_plugin_owned(true),
        ))
        .expect("plugin metadata should be accepted");

    let listed = world
        .list_reflect_types(ReflectSchemaRequest::new(ReflectSchemaFilter {
            include_components: true,
            include_resources: true,
            include_plugin_owned: false,
            ..ReflectSchemaFilter::default()
        }))
        .expect("schema list should route through registry")
        .registrations
        .into_iter()
        .map(|registration| registration.type_path.type_path)
        .collect::<Vec<_>>();
    assert_eq!(
        listed,
        vec!["plugin_a::ProbeComponent", "plugin_a::ProbeResource"]
    );
    assert_eq!(
        world
            .reflect_schema("ProbeResource")
            .expect("short type path should resolve")
            .type_path
            .type_path,
        "plugin_a::ProbeResource"
    );

    let component_address = ReflectObjectAddress::component(entity, "ProbeComponent")
        .expect("component address should be valid");
    let component_fields = world
        .reflect_fields(
            zircon_runtime_interface::reflect::ReflectFieldsRequest::new(component_address.clone()),
        )
        .expect("component fields should route to component adapter");
    assert_eq!(
        component_fields.fields,
        vec![ReflectFieldValue::new(
            "entity",
            ReflectedValue::Unsigned(entity)
        )]
    );
    let component_write = world
        .reflect_write(ReflectWriteRequest::new(
            component_address.clone(),
            "entity",
            ReflectedValue::Unsigned(entity),
        ))
        .expect("component write should route to component adapter and read back");
    assert!(!component_write.changed);
    assert_eq!(
        component_write.field,
        ReflectFieldValue::new("entity", ReflectedValue::Unsigned(entity))
    );

    let resource_address = ReflectObjectAddress::resource("plugin_a::ProbeResource")
        .expect("resource address should be valid");
    let resource_read = world
        .reflect_read(ReflectReadRequest::new(resource_address.clone(), "enabled"))
        .expect("resource read should route to resource adapter");
    assert_eq!(
        resource_read.field,
        ReflectFieldValue::new("enabled", ReflectedValue::Bool(true))
    );
    let resource_write = world
        .reflect_write(ReflectWriteRequest::new(
            resource_address,
            "enabled",
            ReflectedValue::Bool(true),
        ))
        .expect("resource write should route to resource adapter and read back");
    assert!(!resource_write.changed);
    assert_eq!(
        resource_write.field,
        ReflectFieldValue::new("enabled", ReflectedValue::Bool(true))
    );

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(component_address, "missing"))
            .expect_err("unknown field should propagate from adapter"),
        ReflectError::UnknownField {
            type_path: "plugin_a::ProbeComponent".to_string(),
            field_name: "missing".to_string(),
        }
    );
    assert_eq!(
        world
            .reflect_fields(
                zircon_runtime_interface::reflect::ReflectFieldsRequest::new(
                    ReflectObjectAddress::component(entity, "plugin_a::ProbeResource")
                        .expect("address should be valid"),
                )
            )
            .expect_err("resource registration cannot be addressed as component"),
        ReflectError::AddressKindMismatch {
            expected: "component `plugin_a::ProbeResource`".to_string(),
            actual: "non-component `plugin_a::ProbeResource`".to_string(),
        }
    );
    assert_eq!(
        world
            .reflect_fields(
                zircon_runtime_interface::reflect::ReflectFieldsRequest::new(
                    ReflectObjectAddress::component(entity, "plugin_a::PluginHidden")
                        .expect("address should be valid"),
                )
            )
            .expect_err("metadata-only component has no adapter"),
        ReflectError::NoComponentAdapter {
            type_path: "plugin_a::PluginHidden".to_string(),
        }
    );
}
