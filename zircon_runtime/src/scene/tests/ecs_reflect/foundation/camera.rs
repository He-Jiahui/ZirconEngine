use super::*;
use crate::core::framework::render::ProjectionMode;
use crate::scene::components::CameraComponent;

const CAMERA: &str = "zircon_runtime::scene::components::CameraComponent";

#[test]
fn camera_projection_and_ortho_size_reflection_roundtrip() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Camera).unwrap();
    let address = ReflectObjectAddress::component(entity, CAMERA).unwrap();
    let original = world.get::<CameraComponent>(entity).unwrap().clone();
    let schema = world.reflect_schema(CAMERA).unwrap();
    // All other Camera fields retain their skipped reflection policy.
    assert_eq!(
        schema
            .type_info
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        [
            "projection_mode",
            "fov_y_radians",
            "ortho_size",
            "z_near",
            "z_far"
        ]
    );
    for (name, kind, hint) in [
        ("projection_mode", "Enum", ReflectEditorHint::Enum),
        ("ortho_size", "Scalar", ReflectEditorHint::Scalar),
    ] {
        let field = schema
            .type_info
            .fields
            .iter()
            .find(|field| field.name == name)
            .unwrap();
        assert!(field.editable && field.editor_visible);
        assert_eq!(field.value_type_path, kind);
        assert_eq!(field.editor_hint, hint);
    }

    let size = ReflectedValue::Scalar(12.5);
    assert!(
        world
            .reflect_write(ReflectWriteRequest::new(
                address.clone(),
                reflected_field_id(CAMERA, "ortho_size"),
                size.clone(),
            ))
            .unwrap()
            .changed
    );
    for (name, projection) in [
        ("Orthographic", ProjectionMode::Orthographic),
        ("Perspective", ProjectionMode::Perspective),
    ] {
        let value = ReflectedValue::Enum(name.to_string());
        let request = ReflectWriteRequest::new(
            address.clone(),
            reflected_field_id(CAMERA, "projection_mode"),
            value.clone(),
        );
        assert!(world.reflect_write(request.clone()).unwrap().changed);
        assert!(!world.reflect_write(request).unwrap().changed);
        let camera = world.get::<CameraComponent>(entity).unwrap();
        assert_eq!(camera.projection_mode, projection);
        assert_eq!(camera.ortho_size, 12.5);
        assert_eq!(camera.fov_y_radians, original.fov_y_radians);
        assert_eq!(camera.z_near, original.z_near);
        assert_eq!(camera.z_far, original.z_far);
        for (name, expected) in [("projection_mode", value), ("ortho_size", size.clone())] {
            assert_eq!(
                world
                    .reflect_read(ReflectReadRequest::new(
                        address.clone(),
                        reflected_field_id(CAMERA, name),
                    ))
                    .unwrap()
                    .field
                    .value,
                expected
            );
        }
    }
}

#[test]
fn camera_projection_and_ortho_size_reject_invalid_reflected_values() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Camera).unwrap();
    let address = ReflectObjectAddress::component(entity, CAMERA).unwrap();
    let original = world.get::<CameraComponent>(entity).unwrap().clone();
    for value in [
        ReflectedValue::Enum("Oblique".to_string()),
        ReflectedValue::String("Orthographic".to_string()),
    ] {
        let error = world
            .reflect_write(ReflectWriteRequest::new(
                address.clone(),
                reflected_field_id(CAMERA, "projection_mode"),
                value,
            ))
            .unwrap_err();
        assert!(
            matches!(&error, ReflectError::TypeMismatch { type_path, field_name, .. }
                if type_path == CAMERA && field_name == "projection_mode"),
            "{error:?}"
        );
        assert_eq!(world.get::<CameraComponent>(entity), Some(&original));
    }
    // The World entry rejects nonfinite values before invoking the field converter.
    for value in [f32::NAN, f32::INFINITY] {
        let error = world
            .reflect_write(ReflectWriteRequest::new(
                address.clone(),
                reflected_field_id(CAMERA, "ortho_size"),
                ReflectedValue::Scalar(value),
            ))
            .unwrap_err();
        assert!(
            matches!(&error, ReflectError::InvalidValue { type_path, field_name, reason }
                if type_path == CAMERA && field_name == "ortho_size" && reason.contains("not finite")),
            "{error:?}"
        );
        assert_eq!(world.get::<CameraComponent>(entity), Some(&original));
    }
}
