use super::*;

#[test]
fn updated_transform_is_reflected_in_render_extract() {
    let mut world = World::new();
    let cube = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Cube))
        .unwrap()
        .id;
    world
        .update_transform(cube, Transform::from_translation(Vec3::new(2.0, 3.0, 4.0)))
        .unwrap();

    let snapshot = world.to_render_extract();
    let mesh_snapshot = snapshot
        .scene
        .meshes
        .iter()
        .find(|mesh_snapshot| mesh_snapshot.node_id == cube)
        .unwrap();
    assert_eq!(
        mesh_snapshot.transform.translation,
        Vec3::new(2.0, 3.0, 4.0)
    );
}

#[test]
fn mesh_renderer_sort_fields_feed_geometry_phase_queue() {
    let mut world = World::new();
    let render_first = world.spawn_node(NodeKind::Mesh);
    let depth_earlier = world.spawn_node(NodeKind::Mesh);
    let depth_later = world.spawn_node(NodeKind::Mesh);
    let order_middle = world.spawn_node(NodeKind::Mesh);
    let material_later = world.spawn_node(NodeKind::Mesh);

    let fixtures = [
        (render_first, -10, 0, 90, 0.0),
        (depth_earlier, 0, 0, 0, 2.0),
        (depth_later, 0, 0, 0, 0.0),
        (order_middle, 0, 0, 10, 0.0),
        (material_later, 0, 5, -99, 0.0),
    ];

    for (entity, render_queue, material_queue, order_in_layer, depth_bias) in fixtures {
        let mesh = world.get_mut::<MeshRenderer>(entity).unwrap();
        mesh.material_alpha_mode = RenderMaterialAlphaMode::Blend;
        mesh.render_queue = render_queue;
        mesh.material_queue = material_queue;
        mesh.order_in_layer = order_in_layer;
        mesh.depth_bias = depth_bias;
        world
            .update_transform(
                entity,
                Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
            )
            .unwrap();
    }

    let extract = world.to_render_frame_extract();
    let mesh_index = |entity| {
        extract
            .geometry
            .meshes
            .iter()
            .position(|mesh| mesh.node_id == entity)
            .expect("mesh should be extracted")
    };
    let render_first_index = mesh_index(render_first);
    let depth_earlier_index = mesh_index(depth_earlier);
    let depth_later_index = mesh_index(depth_later);
    let order_middle_index = mesh_index(order_middle);
    let material_later_index = mesh_index(material_later);

    for (entity, render_queue, material_queue, order_in_layer, depth_bias) in fixtures {
        let mesh_index = mesh_index(entity);
        let input = extract
            .geometry
            .phase_inputs
            .iter()
            .find(|input| input.entity == entity && input.mesh_index == mesh_index)
            .expect("phase input should carry mesh sort fields");
        assert_eq!(input.render_queue, render_queue);
        assert_eq!(input.material_queue, material_queue);
        assert_eq!(input.order_in_layer, order_in_layer);
        assert_eq!(input.depth_bias, depth_bias);
    }

    assert_eq!(
        extract
            .geometry
            .phase_queue
            .items_for_phase(RenderPhase::Transparent3d)
            .filter(|item| {
                [
                    render_first,
                    depth_earlier,
                    depth_later,
                    order_middle,
                    material_later,
                ]
                .contains(&item.entity)
            })
            .map(|item| item.mesh_source)
            .collect::<Vec<_>>(),
        vec![
            RenderPhaseMeshSource::MeshIndex(render_first_index),
            RenderPhaseMeshSource::MeshIndex(depth_earlier_index),
            RenderPhaseMeshSource::MeshIndex(depth_later_index),
            RenderPhaseMeshSource::MeshIndex(order_middle_index),
            RenderPhaseMeshSource::MeshIndex(material_later_index),
        ]
    );
}

#[test]
fn render_mesh_phase_projection_consumes_entries_without_cloning_snapshots() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("render.rs"),
    )
    .unwrap();
    let projection = source
        .split("fn collect_render_meshes_and_phase_inputs")
        .nth(1)
        .and_then(|text| {
            text.split("fn material_property_overrides_for_meshes")
                .next()
        })
        .expect("read mesh/phase projection helper");

    assert!(
        !projection.contains("(*mesh).clone()"),
        "per-frame mesh/phase projection must consume each prepared snapshot instead of deep-cloning it into a second Vec"
    );
    assert!(
        source.contains("visit_render_mesh_snapshots_for_camera")
            && !source.contains("fn render_mesh_snapshots_for_camera"),
        "mesh extraction must push snapshots directly into caller-owned buffers instead of allocating a temporary Vec per entity"
    );
    assert!(
        !source.contains("fn material_property_overrides_for_meshes"),
        "visible mesh overrides must be captured once while their MeshRenderer is already borrowed, not re-looked-up and cloned once per primitive"
    );
    let snapshot_visitor = source
        .split("fn visit_render_mesh_snapshots_for_camera")
        .nth(1)
        .and_then(|text| text.split("fn collect_render_sprites").next())
        .expect("read render mesh snapshot visitor");
    assert_eq!(
        snapshot_visitor
            .matches("render_mesh_transform_revision(&transform)")
            .count(),
        1,
        "a multi-primitive mesh must hash its shared transform once per entity, not once per primitive"
    );
}

#[test]
fn render_extract_separates_directional_point_and_spot_lights() {
    let mut world = World::new();
    let point = world.spawn_node(NodeKind::PointLight);
    let spot = world.spawn_node(NodeKind::SpotLight);

    world
        .update_transform(point, Transform::from_translation(Vec3::new(3.0, 4.0, 5.0)))
        .unwrap();
    world
        .update_transform(spot, Transform::from_translation(Vec3::new(-2.0, 6.0, 1.5)))
        .unwrap();

    world
        .set_property(
            point,
            &ComponentPropertyPath::parse("PointLight.color").unwrap(),
            ScenePropertyValue::Vec3([0.2, 0.4, 0.8]),
        )
        .unwrap();
    world
        .set_property(
            point,
            &ComponentPropertyPath::parse("PointLight.intensity").unwrap(),
            ScenePropertyValue::Scalar(6.5),
        )
        .unwrap();
    world
        .set_property(
            point,
            &ComponentPropertyPath::parse("PointLight.range").unwrap(),
            ScenePropertyValue::Scalar(9.0),
        )
        .unwrap();

    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.direction").unwrap(),
            ScenePropertyValue::Vec3([0.0, -1.0, 0.25]),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.color").unwrap(),
            ScenePropertyValue::Vec3([1.0, 0.8, 0.3]),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.intensity").unwrap(),
            ScenePropertyValue::Scalar(12.0),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.range").unwrap(),
            ScenePropertyValue::Scalar(15.0),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.inner_angle_radians").unwrap(),
            ScenePropertyValue::Scalar(0.35),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.outer_angle_radians").unwrap(),
            ScenePropertyValue::Scalar(0.65),
        )
        .unwrap();

    let snapshot = world.to_render_extract();

    assert_eq!(snapshot.scene.directional_lights.len(), 1);

    let point_light = snapshot
        .scene
        .point_lights
        .iter()
        .find(|light| light.node_id == point)
        .unwrap();
    assert_eq!(point_light.position, Vec3::new(3.0, 4.0, 5.0));
    assert_eq!(point_light.color, Vec3::new(0.2, 0.4, 0.8));
    assert_eq!(point_light.intensity, 6.5);
    assert_eq!(point_light.range, 9.0);

    let spot_light = snapshot
        .scene
        .spot_lights
        .iter()
        .find(|light| light.node_id == spot)
        .unwrap();
    assert_eq!(spot_light.position, Vec3::new(-2.0, 6.0, 1.5));
    assert_eq!(spot_light.direction, Vec3::new(0.0, -1.0, 0.25));
    assert_eq!(spot_light.color, Vec3::new(1.0, 0.8, 0.3));
    assert_eq!(spot_light.intensity, 12.0);
    assert_eq!(spot_light.range, 15.0);
    assert_eq!(spot_light.inner_angle_radians, 0.35);
    assert_eq!(spot_light.outer_angle_radians, 0.65);

    let frame_extract = world.to_render_frame_extract();
    assert_eq!(frame_extract.lighting.directional_lights.len(), 1);
    assert!(
        frame_extract
            .lighting
            .point_lights
            .iter()
            .any(|light| light.node_id == point)
    );
    assert!(
        frame_extract
            .lighting
            .spot_lights
            .iter()
            .any(|light| light.node_id == spot)
    );
}

#[test]
fn render_product_pbr_world_frame_extract_exposes_authored_ambient_and_rect_light_slots() {
    let mut world = World::new();
    let ambient = world.spawn_node(NodeKind::AmbientLight);
    let rect = world.spawn_node(NodeKind::RectLight);

    world
        .update_transform(
            rect,
            Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)).with_rotation(
                crate::core::math::Quat::from_rotation_y(45.0_f32.to_radians()),
            ),
        )
        .unwrap();
    world
        .set_property(
            ambient,
            &ComponentPropertyPath::parse("AmbientLight.color").unwrap(),
            ScenePropertyValue::Vec3([0.05, 0.06, 0.07]),
        )
        .unwrap();
    world
        .set_property(
            ambient,
            &ComponentPropertyPath::parse("AmbientLight.intensity").unwrap(),
            ScenePropertyValue::Scalar(0.35),
        )
        .unwrap();
    world
        .set_property(
            ambient,
            &ComponentPropertyPath::parse("AmbientLight.affects_lightmapped_meshes").unwrap(),
            ScenePropertyValue::Bool(false),
        )
        .unwrap();
    world
        .set_property(
            rect,
            &ComponentPropertyPath::parse("RectLight.color").unwrap(),
            ScenePropertyValue::Vec3([1.0, 0.8, 0.6]),
        )
        .unwrap();
    world
        .set_property(
            rect,
            &ComponentPropertyPath::parse("RectLight.intensity").unwrap(),
            ScenePropertyValue::Scalar(12.0),
        )
        .unwrap();
    world
        .set_property(
            rect,
            &ComponentPropertyPath::parse("RectLight.range").unwrap(),
            ScenePropertyValue::Scalar(16.0),
        )
        .unwrap();
    world
        .set_property(
            rect,
            &ComponentPropertyPath::parse("RectLight.size").unwrap(),
            ScenePropertyValue::Vec2([2.0, 0.5]),
        )
        .unwrap();

    let snapshot = world.to_render_extract();
    assert_eq!(snapshot.scene.ambient_lights.len(), 1);
    assert_eq!(
        snapshot.scene.ambient_lights[0].color,
        Vec3::new(0.05, 0.06, 0.07)
    );
    assert_eq!(snapshot.scene.ambient_lights[0].intensity, 0.35);
    assert!(!snapshot.scene.ambient_lights[0].renderer_degraded);
    assert_eq!(snapshot.scene.ambient_lights[0].degradation_reason, None);

    let rect_light = snapshot
        .scene
        .rect_lights
        .iter()
        .find(|light| light.node_id == rect)
        .unwrap();
    assert_eq!(rect_light.position, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(
        rect_light.direction,
        world.world_transform(rect).unwrap().forward()
    );
    assert_eq!(rect_light.color, Vec3::new(1.0, 0.8, 0.6));
    assert_eq!(rect_light.intensity, 12.0);
    assert_eq!(rect_light.range, 16.0);
    assert_eq!(rect_light.size, Vec2::new(2.0, 0.5));
    assert!(rect_light.renderer_degraded);

    let extract = world.to_render_frame_extract();
    assert_eq!(
        extract.lighting.ambient_lights,
        snapshot.scene.ambient_lights
    );
    assert_eq!(extract.lighting.rect_lights, snapshot.scene.rect_lights);

    let default_ambient = RenderAmbientLightSnapshot::default();
    assert!(default_ambient.renderer_degraded);
    assert!(
        default_ambient
            .degradation_reason
            .as_deref()
            .unwrap()
            .contains("no authored scene component")
    );
}
