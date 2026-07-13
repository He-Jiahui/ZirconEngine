use super::*;

#[test]
fn scene_property_values_convert_to_reflected_values() {
    let cases = [
        (ScenePropertyValue::Bool(true), ReflectedValue::Bool(true)),
        (ScenePropertyValue::Integer(-7), ReflectedValue::Integer(-7)),
        (ScenePropertyValue::Unsigned(9), ReflectedValue::Unsigned(9)),
        (ScenePropertyValue::Scalar(1.5), ReflectedValue::Scalar(1.5)),
        (
            ScenePropertyValue::String("name".to_string()),
            ReflectedValue::String("name".to_string()),
        ),
        (
            ScenePropertyValue::Enum("Dynamic".to_string()),
            ReflectedValue::Enum("Dynamic".to_string()),
        ),
        (
            ScenePropertyValue::Vec2([1.0, 2.0]),
            ReflectedValue::Vec2([1.0, 2.0]),
        ),
        (
            ScenePropertyValue::Vec3([1.0, 2.0, 3.0]),
            ReflectedValue::Vec3([1.0, 2.0, 3.0]),
        ),
        (
            ScenePropertyValue::Vec4([1.0, 2.0, 3.0, 4.0]),
            ReflectedValue::Vec4([1.0, 2.0, 3.0, 4.0]),
        ),
        (
            ScenePropertyValue::Quaternion([0.0, 0.0, 0.0, 1.0]),
            ReflectedValue::Quaternion([0.0, 0.0, 0.0, 1.0]),
        ),
        (
            ScenePropertyValue::Entity(Some(42)),
            ReflectedValue::Entity(Some(42)),
        ),
        (
            ScenePropertyValue::Entity(None),
            ReflectedValue::Entity(None),
        ),
        (
            ScenePropertyValue::Resource("mesh://cube".to_string()),
            ReflectedValue::Resource("mesh://cube".to_string()),
        ),
    ];

    for (scene_value, reflected_value) in cases {
        assert_eq!(
            reflected_from_scene_value(scene_value).expect("scene value should convert"),
            reflected_value
        );
    }
}

#[test]
fn reflected_values_convert_to_scene_property_values_when_supported() {
    let cases = [
        (ReflectedValue::Bool(true), ScenePropertyValue::Bool(true)),
        (ReflectedValue::Integer(-7), ScenePropertyValue::Integer(-7)),
        (ReflectedValue::Unsigned(9), ScenePropertyValue::Unsigned(9)),
        (ReflectedValue::Scalar(1.5), ScenePropertyValue::Scalar(1.5)),
        (
            ReflectedValue::String("name".to_string()),
            ScenePropertyValue::String("name".to_string()),
        ),
        (
            ReflectedValue::Enum("Dynamic".to_string()),
            ScenePropertyValue::Enum("Dynamic".to_string()),
        ),
        (
            ReflectedValue::Vec2([1.0, 2.0]),
            ScenePropertyValue::Vec2([1.0, 2.0]),
        ),
        (
            ReflectedValue::Vec3([1.0, 2.0, 3.0]),
            ScenePropertyValue::Vec3([1.0, 2.0, 3.0]),
        ),
        (
            ReflectedValue::Vec4([1.0, 2.0, 3.0, 4.0]),
            ScenePropertyValue::Vec4([1.0, 2.0, 3.0, 4.0]),
        ),
        (
            ReflectedValue::Quaternion([0.0, 0.0, 0.0, 1.0]),
            ScenePropertyValue::Quaternion([0.0, 0.0, 0.0, 1.0]),
        ),
        (
            ReflectedValue::Entity(Some(42)),
            ScenePropertyValue::Entity(Some(42)),
        ),
        (
            ReflectedValue::Entity(None),
            ScenePropertyValue::Entity(None),
        ),
        (
            ReflectedValue::Resource("mesh://cube".to_string()),
            ScenePropertyValue::Resource("mesh://cube".to_string()),
        ),
    ];

    for (reflected_value, scene_value) in cases {
        assert_eq!(
            scene_value_from_reflected(reflected_value).expect("reflected value should convert"),
            scene_value
        );
    }

    assert_eq!(
        scene_value_from_reflected(ReflectedValue::Null).expect_err("null is not a scene value"),
        ReflectError::UnsupportedConversion {
            source: "ReflectedValue::Null".to_string(),
            target: "ScenePropertyValue".to_string(),
        }
    );
    assert!(matches!(
        scene_value_from_reflected(ReflectedValue::List(Vec::new())),
        Err(ReflectError::UnsupportedConversion { .. })
    ));
    assert!(matches!(
        scene_value_from_reflected(ReflectedValue::Map(BTreeMap::new())),
        Err(ReflectError::UnsupportedConversion { .. })
    ));
    assert!(matches!(
        scene_value_from_reflected(ReflectedValue::Json(json!({ "arbitrary": true }))),
        Err(ReflectError::UnsupportedConversion { .. })
    ));
    assert!(matches!(
        scene_value_from_reflected(ReflectedValue::Scalar(f32::NAN)),
        Err(ReflectError::UnsupportedConversion { .. })
    ));
    assert!(matches!(
        scene_value_from_reflected(ReflectedValue::Quaternion([0.0, 0.0, f32::INFINITY, 1.0])),
        Err(ReflectError::UnsupportedConversion { .. })
    ));
}

#[test]
fn animation_parameter_conversion_returns_structured_error() {
    assert_eq!(
        reflected_from_scene_value(ScenePropertyValue::AnimationParameter(
            AnimationParameterValue::Bool(true),
        ))
        .expect_err("animation parameters are outside M8.3 value conversion"),
        ReflectError::UnsupportedConversion {
            source: "ScenePropertyValue::AnimationParameter".to_string(),
            target: "ReflectedValue".to_string(),
        }
    );
}
