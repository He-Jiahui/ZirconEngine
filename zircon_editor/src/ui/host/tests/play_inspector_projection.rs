use std::time::{Duration, Instant};

use zircon_runtime_interface::reflect::ReflectedValue;
use zircon_runtime_interface::world_sync::{WorldInspectionFieldRow, WorldQueryResult};
use zircon_runtime_interface::{GatewaySessionIdentity, ZrRuntimeSessionHandle};

use super::{PlayInspectorProjection, PLAY_INSPECTOR_QUERY_INTERVAL};

#[test]
fn actual_runtime_query_preserves_native_fields_in_play_projection() {
    let scene = zircon_runtime::scene::Scene::new();
    for node in scene.nodes() {
        let mut projection = PlayInspectorProjection::default();
        let identity = identity(1);
        let result = scene.query_world(
            &zircon_runtime_interface::world_sync::WorldQuery::inspection_fields(node.id, None),
        );
        projection
            .apply(identity.clone(), node.id, result, None)
            .unwrap();
        let snapshot = projection.snapshot_for(&identity, node.id).unwrap();
        let source = scene.inspection_fields_artifact(node.id).unwrap();
        assert_eq!(
            snapshot.native_fields,
            crate::ui::workbench::snapshot::InspectorNativeFieldSnapshot::project(source.fields())
        );
        assert!(!snapshot.native_fields.is_empty());
        assert!(snapshot.plugin_components.is_empty());
    }
}

fn identity(gateway_generation: u64) -> GatewaySessionIdentity {
    GatewaySessionIdentity::new(3, ZrRuntimeSessionHandle::new(5), 7, None)
        .with_play_instance(Some(11))
        .with_gateway_generation(gateway_generation)
}

fn field(
    component_type_path: &str,
    component_display_name: &str,
    field_name: &str,
    value: ReflectedValue,
    plugin_owned: bool,
) -> WorldInspectionFieldRow {
    WorldInspectionFieldRow {
        component_type_path: component_type_path.to_string(),
        component_display_name: component_display_name.to_string(),
        field_name: field_name.to_string(),
        field_display_name: field_name.to_string(),
        value_type_path: value.type_name().to_string(),
        value,
        writable: true,
        serializable: true,
        plugin_owned,
    }
}

#[test]
fn focused_query_cadence_is_immediate_then_generation_qualified() {
    let mut projection = PlayInspectorProjection::default();
    let identity = identity(1);
    let started = Instant::now();

    assert_eq!(projection.begin_query(&identity, 7, started), Some(None));
    projection
        .apply(
            identity.clone(),
            7,
            WorldQueryResult::InspectionFields {
                generation: 4,
                entity: 7,
                fields: Vec::new(),
            },
            None,
        )
        .expect("first focused projection should be valid");
    assert_eq!(
        projection.begin_query(
            &identity,
            7,
            started + PLAY_INSPECTOR_QUERY_INTERVAL - Duration::from_millis(1)
        ),
        None
    );
    assert_eq!(
        projection.begin_query(&identity, 7, started + PLAY_INSPECTOR_QUERY_INTERVAL),
        Some(Some(4))
    );
}

#[test]
fn play_snapshot_projects_runtime_values_and_writable_plugin_fields() {
    let mut projection = PlayInspectorProjection::default();
    let identity = identity(1);
    let fields = vec![
        field(
            "zircon_runtime::scene::components::Name",
            "Name",
            "value",
            ReflectedValue::String("Runtime Hero".to_string()),
            false,
        ),
        field(
            "zircon_runtime::scene::components::LocalTransform",
            "Transform",
            "translation",
            ReflectedValue::Vec3([1.0, 2.0, 3.0]),
            false,
        ),
        field(
            "zircon_runtime::scene::components::LocalTransform",
            "Transform",
            "scale",
            ReflectedValue::Vec3([4.0, 5.0, 6.0]),
            false,
        ),
        field(
            "zircon_runtime::scene::components::LocalTransform",
            "Transform",
            "rotation",
            ReflectedValue::Quaternion(
                zircon_runtime_interface::math::Quat::from_rotation_x(30_f32.to_radians())
                    .to_array(),
            ),
            false,
        ),
        field(
            "weather.cloud_layer",
            "Cloud Layer",
            "coverage",
            ReflectedValue::Scalar(0.75),
            true,
        ),
    ];

    assert!(projection
        .apply(
            identity.clone(),
            7,
            WorldQueryResult::InspectionFields {
                generation: 4,
                entity: 7,
                fields: fields.clone(),
            },
            None,
        )
        .expect("runtime Inspector fields should project"));
    let snapshot = projection
        .snapshot_for(&identity, 7)
        .expect("matching identity/entity should expose the runtime Inspector");
    assert_eq!(snapshot.name, "Runtime Hero");
    assert_eq!(snapshot.translation, ["1.00", "2.00", "3.00"]);
    assert_eq!(snapshot.scale, ["4.00", "5.00", "6.00"]);
    assert_eq!(
        snapshot.rotation_degrees,
        Some(["30.00".to_string(), "0.00".to_string(), "0.00".to_string()])
    );
    assert_eq!(snapshot.plugin_components.len(), 1);
    assert!(snapshot.plugin_components[0].properties[0].editable);
    assert!(!projection
        .apply(
            identity,
            7,
            WorldQueryResult::InspectionFields {
                generation: 5,
                entity: 7,
                fields,
            },
            None,
        )
        .expect("an unchanged visible Inspector may still advance its generation"));
}

#[test]
fn actual_world_rotation_inspection_projects_read_only_xyz_degrees() {
    use zircon_runtime::scene::{NodeKind, World};
    use zircon_runtime_interface::math::{EulerRot, Quat, Transform};

    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("real entity should spawn");
    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        30_f32.to_radians(),
        10_f32.to_radians(),
        -20_f32.to_radians(),
    );
    world
        .update_transform(
            entity,
            Transform {
                rotation,
                ..Transform::default()
            },
        )
        .expect("real transform should update");
    let inspection = world
        .inspection_fields_artifact(entity)
        .expect("real world should produce inspection fields");
    let rotation_field = inspection
        .fields()
        .iter()
        .find(|field| {
            field.component_type_path == super::LOCAL_TRANSFORM_COMPONENT_TYPE_PATH
                && field.field_name == "rotation"
        })
        .expect("actual LocalTransform rotation field");
    assert_eq!(
        rotation_field.value,
        ReflectedValue::Vec4(rotation.to_array())
    );
    assert!(!rotation_field.writable);
    assert!(!rotation_field.plugin_owned);
    let hierarchy = world.inspection_artifact();
    let identity = identity(1);
    let mut projection = PlayInspectorProjection::default();
    projection
        .apply(
            identity.clone(),
            entity,
            WorldQueryResult::InspectionFields {
                generation: inspection.generation(),
                entity,
                fields: inspection.fields().to_vec(),
            },
            hierarchy.hierarchy_row(entity),
        )
        .expect("actual inspection fields should project");
    let snapshot = projection
        .snapshot_for(&identity, entity)
        .expect("selected runtime snapshot");
    assert_eq!(
        snapshot.rotation_degrees,
        Some(["30.00".to_owned(), "10.00".to_owned(), "-20.00".to_owned()])
    );
    assert_eq!(
        world
            .local_transform(entity)
            .expect("unchanged real transform")
            .rotation,
        rotation
    );
}

#[test]
fn stale_identity_snapshot_is_never_exposed_to_a_replacement_runtime() {
    let mut projection = PlayInspectorProjection::default();
    let original = identity(1);
    projection
        .apply(
            original.clone(),
            7,
            WorldQueryResult::InspectionFields {
                generation: 4,
                entity: 7,
                fields: Vec::new(),
            },
            None,
        )
        .expect("base Inspector should project");

    assert!(projection.snapshot_for(&identity(2), 7).is_none());
    assert!(projection.snapshot_for(&original, 8).is_none());
}
