use super::*;

#[test]
fn ambient_and_rect_light_reflection_roundtrips_authoring_fields() {
    let mut world = World::empty();
    let ambient = world
        .spawn_node(NodeKind::AmbientLight)
        .expect("test scene spawn should succeed");
    let rect = world
        .spawn_node(NodeKind::RectLight)
        .expect("test scene spawn should succeed");
    let ambient_address = fixed_component_address(ambient, "AmbientLight");
    let rect_address = fixed_component_address(rect, "RectLight");

    world
        .reflect_write(ReflectWriteRequest::new(
            ambient_address.clone(),
            reflected_field_id("zircon_runtime::scene::components::AmbientLight", "color"),
            ReflectedValue::Vec3([0.05, 0.06, 0.07]),
        ))
        .expect("ambient color should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            ambient_address.clone(),
            reflected_field_id(
                "zircon_runtime::scene::components::AmbientLight",
                "intensity",
            ),
            ReflectedValue::Scalar(0.35),
        ))
        .expect("ambient intensity should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            ambient_address.clone(),
            reflected_field_id(
                "zircon_runtime::scene::components::AmbientLight",
                "affects_lightmapped_meshes",
            ),
            ReflectedValue::Bool(false),
        ))
        .expect("ambient lightmap flag should be writable");

    let ambient_light = world.get::<AmbientLight>(ambient).unwrap();
    assert_eq!(ambient_light.color, Vec3::new(0.05, 0.06, 0.07));
    assert_eq!(ambient_light.intensity, 0.35);
    assert!(!ambient_light.affects_lightmapped_meshes);
    assert!(world
        .reflect_fields(
            zircon_runtime_interface::reflect::ReflectFieldsRequest::new(ambient_address)
        )
        .expect("ambient fields should be enumerable")
        .fields
        .contains(&ReflectFieldValue::new(
            reflected_field_id(
                "zircon_runtime::scene::components::AmbientLight",
                "affects_lightmapped_meshes",
            ),
            "affects_lightmapped_meshes",
            ReflectedValue::Bool(false)
        )));

    world
        .reflect_write(ReflectWriteRequest::new(
            rect_address.clone(),
            reflected_field_id("zircon_runtime::scene::components::RectLight", "range"),
            ReflectedValue::Scalar(16.0),
        ))
        .expect("rect range should be writable");
    world
        .reflect_write(ReflectWriteRequest::new(
            rect_address.clone(),
            reflected_field_id("zircon_runtime::scene::components::RectLight", "size"),
            ReflectedValue::Vec2([2.0, 0.5]),
        ))
        .expect("rect size should be writable");

    let rect_light = world.get::<RectLight>(rect).unwrap();
    assert_eq!(rect_light.range, 16.0);
    assert_eq!(rect_light.size, Vec2::new(2.0, 0.5));
    assert!(world
        .reflect_fields(zircon_runtime_interface::reflect::ReflectFieldsRequest::new(rect_address))
        .expect("rect fields should be enumerable")
        .fields
        .contains(&ReflectFieldValue::new(
            reflected_field_id("zircon_runtime::scene::components::RectLight", "size"),
            "size",
            ReflectedValue::Vec2([2.0, 0.5])
        )));
}

#[test]
fn name_component_reads_and_writes_through_world_reflection() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let address =
        ReflectObjectAddress::component(entity, "zircon_runtime::scene::components::Name")
            .expect("fixed component full-path address");

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(
                address.clone(),
                reflected_field_id("zircon_runtime::scene::components::Name", "value"),
            ))
            .expect("name should be readable")
            .field,
        ReflectFieldValue::new(
            reflected_field_id("zircon_runtime::scene::components::Name", "value"),
            "value",
            ReflectedValue::String("Mesh 1".to_string()),
        )
    );

    let response = world
        .reflect_write(ReflectWriteRequest::new(
            address,
            reflected_field_id("zircon_runtime::scene::components::Name", "value"),
            ReflectedValue::String("Reflected Name".to_string()),
        ))
        .expect("name should be writable");

    assert!(response.changed);
    assert_eq!(
        response.field,
        ReflectFieldValue::new(
            reflected_field_id("zircon_runtime::scene::components::Name", "value"),
            "value",
            ReflectedValue::String("Reflected Name".to_string())
        )
    );
    assert_eq!(world.get::<Name>(entity).unwrap().0, "Reflected Name");
}
